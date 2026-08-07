mod bridge;
mod metrics;
mod peer;
mod ports;

pub use metrics::{MessageMetrics, MessageMetricsSnapshot, TopicSnapshot};
pub use peer::MessagePeer;
pub use ports::{
    DEFAULT_RCV_HWM, DEFAULT_SND_HWM, DEFAULT_XPUB_BIND, DEFAULT_XSUB_BIND, XPUB_CHANNEL,
    XPUB_PORT, XSUB_CHANNEL, XSUB_PORT,
};

use crate::shutdown;
use crate::transports::{bind_all, format_endpoints};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use zmq::{Context as ZmqContext, Socket, SocketType};

const PROXY_CONTROL: &str = "inproc://robot_bus/message_bus/proxy-ctl";
const PROXY_CAPTURE: &str = "inproc://robot_bus/message_bus/proxy-capture";

#[derive(Clone, Debug)]
pub struct BusConfig {
    pub xsub_bind: String,
    pub xpub_bind: String,
    pub snd_hwm: i32,
    pub rcv_hwm: i32,
    /// When true (default), also bind inproc + ipc aliases via [`bind_all`].
    pub bind_all_transports: bool,
    /// Stable id for hop-path loop prevention. Empty → random UUID at start.
    pub broker_id: String,
    /// Static peers for topic federation (empty → plain `proxy_steerable`).
    pub peers: Vec<MessagePeer>,
}

impl Default for BusConfig {
    fn default() -> Self {
        Self {
            xsub_bind: DEFAULT_XSUB_BIND.to_string(),
            xpub_bind: DEFAULT_XPUB_BIND.to_string(),
            snd_hwm: DEFAULT_SND_HWM,
            rcv_hwm: DEFAULT_RCV_HWM,
            bind_all_transports: true,
            broker_id: String::new(),
            peers: Vec::new(),
        }
    }
}

/// Run a ZeroMQ XSUB/XPUB proxy until `shutdown` is set.
///
/// Forwarding always uses libzmq [`zmq::proxy_steerable`] — the same data path as
/// before monitoring. When `metrics` is set, a capture socket mirrors traffic to a
/// **side thread** for counting; the forward path stays in C.
///
/// Metrics only see messages that actually flow (real subscribers present). No
/// internal blanket SUB is attached — that would force every publish through the
/// bus and hurt communication efficiency.
///
/// When [`BusConfig::peers`] is non-empty, a custom federated forwarder is used
/// instead (hop-path anti-loop + on-demand topic push).
pub fn run_with_shutdown(
    config: BusConfig,
    shutdown: Arc<AtomicBool>,
    metrics: Option<Arc<MessageMetrics>>,
) -> Result<()> {
    run_with_shutdown_ctx(ZmqContext::new(), config, shutdown, metrics)
}

/// Like [`run_with_shutdown`], but sockets are created from the given context
/// (required for same-process `inproc://` with SDK participants).
pub fn run_with_shutdown_ctx(
    context: ZmqContext,
    config: BusConfig,
    shutdown: Arc<AtomicBool>,
    metrics: Option<Arc<MessageMetrics>>,
) -> Result<()> {
    let xsub = context
        .socket(SocketType::XSUB)
        .context("create XSUB socket")?;
    let xpub = context
        .socket(SocketType::XPUB)
        .context("create XPUB socket")?;

    apply_low_latency_options(&xsub, config.snd_hwm, config.rcv_hwm)?;
    apply_low_latency_options(&xpub, config.snd_hwm, config.rcv_hwm)?;

    let (xsub_endpoints, xpub_endpoints) = if config.bind_all_transports {
        (
            bind_all(&xsub, &config.xsub_bind, ports::XSUB_CHANNEL)?,
            bind_all(&xpub, &config.xpub_bind, ports::XPUB_CHANNEL)?,
        )
    } else {
        xsub.bind(&config.xsub_bind)
            .with_context(|| format!("bind {}", config.xsub_bind))?;
        xpub.bind(&config.xpub_bind)
            .with_context(|| format!("bind {}", config.xpub_bind))?;
        (
            vec![config.xsub_bind.clone()],
            vec![config.xpub_bind.clone()],
        )
    };

    let federated = !config.peers.is_empty();
    println!(
        "message_bus_broker proxy started\n  \
         publishers (PUB) connect ->\n    {}\n  \
         subscribers (SUB) connect ->\n    {}\n  \
         transports: {}\n  \
         forwarding: {}",
        format_endpoints(&xsub_endpoints),
        format_endpoints(&xpub_endpoints),
        if config.bind_all_transports {
            "tcp + inproc + ipc per socket"
        } else {
            "tcp only"
        },
        if federated {
            "federated forwarder (opaque multipart + hop-path peers)"
        } else if metrics.is_some() {
            "opaque multipart frames (libzmq proxy_steerable + side-thread capture metrics)"
        } else {
            "opaque multipart frames (libzmq proxy_steerable)"
        },
    );

    if federated {
        return bridge::run_federated(&context, xsub, xpub, &config, &shutdown, metrics);
    }

    let control = context
        .socket(SocketType::PAIR)
        .context("create proxy control PAIR")?;
    let control_client = context
        .socket(SocketType::PAIR)
        .context("create proxy control client")?;

    control.bind(PROXY_CONTROL).context("bind proxy control")?;
    control_client
        .connect(PROXY_CONTROL)
        .context("connect proxy control")?;

    let mut xsub = xsub;
    let mut xpub = xpub;
    let mut control = control;

    if let Some(metrics) = metrics {
        // Capture mirrors traffic to a side thread. High HWM so metrics never
        // back-pressure the C proxy (drops are OK — monitoring may under-count).
        let mut capture = context
            .socket(SocketType::PAIR)
            .context("create proxy capture PAIR")?;
        capture.bind(PROXY_CAPTURE).context("bind proxy capture")?;
        capture.set_sndhwm(10_000).context("capture sndhwm")?;
        capture.set_linger(0).context("capture linger")?;

        let reader = context
            .socket(SocketType::PAIR)
            .context("create capture reader")?;
        reader
            .connect(PROXY_CAPTURE)
            .context("connect capture reader")?;
        reader.set_rcvhwm(10_000).context("reader rcvhwm")?;
        reader.set_linger(0).context("reader linger")?;

        // Keep draining capture until the proxy thread exits — stopping the
        // reader early can stall `proxy_steerable_with_capture` on TERMINATE.
        let capture_stop = Arc::new(AtomicBool::new(false));
        let capture_stop_flag = Arc::clone(&capture_stop);
        let metrics_join =
            thread::spawn(move || capture_metrics_loop(reader, metrics, capture_stop_flag));
        let proxy = thread::spawn(move || {
            if let Err(err) =
                zmq::proxy_steerable_with_capture(&mut xsub, &mut xpub, &mut capture, &mut control)
            {
                log::warn!("message bus proxy stopped with error: {err}");
            }
        });

        wait_until_shutdown(&shutdown);
        let _ = control_client.send(b"TERMINATE".as_ref(), 0);
        proxy
            .join()
            .map_err(|e| anyhow::anyhow!("message bus proxy thread: {e:?}"))?;
        capture_stop.store(true, Ordering::Release);
        let _ = metrics_join.join();
    } else {
        let proxy = thread::spawn(move || {
            let _ = zmq::proxy_steerable(&mut xsub, &mut xpub, &mut control);
        });

        wait_until_shutdown(&shutdown);
        let _ = control_client.send(b"TERMINATE".as_ref(), 0);
        proxy
            .join()
            .map_err(|e| anyhow::anyhow!("message bus proxy thread: {e:?}"))?;
    }

    Ok(())
}

fn wait_until_shutdown(shutdown: &AtomicBool) {
    while !shutdown.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }
}

/// Side thread: read capture copies and update counters. Never on the forward path.
fn capture_metrics_loop(reader: Socket, metrics: Arc<MessageMetrics>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        let mut items = [reader.as_poll_item(zmq::POLLIN)];
        if zmq::poll(&mut items, 100).is_err() {
            break;
        }
        if !items[0].is_readable() {
            continue;
        }
        while let Ok(frames) = reader.recv_multipart(zmq::DONTWAIT) {
            // Capture mirrors both directions. Payloads are `[topic][body…]`;
            // subscription frames are typically a single 0/1-prefixed frame.
            if frames.len() < 2 {
                continue;
            }
            let Some(topic) = frames.first().and_then(|f| std::str::from_utf8(f).ok()) else {
                continue;
            };
            // Console snapshot / control-plane traffic is internal bookkeeping,
            // not user data — drain it but don't let it pollute topic metrics.
            if topic.starts_with("/_robot_bus/") {
                continue;
            }
            let bytes: u64 = frames.iter().map(|f| f.len() as u64).sum();
            metrics.record(topic, bytes);
        }
    }
}

/// Run until interrupted (installs process Ctrl+C handler).
pub fn run(config: BusConfig) -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    shutdown::install(shutdown.clone());
    run_with_shutdown(config, shutdown, Some(MessageMetrics::new()))
}

fn apply_low_latency_options(socket: &Socket, snd_hwm: i32, rcv_hwm: i32) -> Result<()> {
    socket.set_linger(0).context("set linger")?;
    socket.set_sndhwm(snd_hwm).context("set sndhwm")?;
    socket.set_rcvhwm(rcv_hwm).context("set rcvhwm")?;
    socket.set_immediate(true).context("set immediate")?;
    Ok(())
}
