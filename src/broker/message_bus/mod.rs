mod ports;

pub use ports::{
    DEFAULT_RCV_HWM, DEFAULT_SND_HWM, DEFAULT_XPUB_BIND, DEFAULT_XSUB_BIND, XPUB_CHANNEL,
    XPUB_PORT, XSUB_CHANNEL, XSUB_PORT,
};

use anyhow::{Context, Result};
use crate::shutdown;
use crate::transports::{bind_all, format_endpoints};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use zmq::{Context as ZmqContext, Socket, SocketType};

const PROXY_CONTROL: &str = "inproc://robot_bus/message_bus/proxy-ctl";

#[derive(Clone, Debug)]
pub struct BusConfig {
    pub xsub_bind: String,
    pub xpub_bind: String,
    pub snd_hwm: i32,
    pub rcv_hwm: i32,
}

impl Default for BusConfig {
    fn default() -> Self {
        Self {
            xsub_bind: DEFAULT_XSUB_BIND.to_string(),
            xpub_bind: DEFAULT_XPUB_BIND.to_string(),
            snd_hwm: DEFAULT_SND_HWM,
            rcv_hwm: DEFAULT_RCV_HWM,
        }
    }
}

/// Run a transparent ZeroMQ XSUB/XPUB proxy until `shutdown` is set.
pub fn run_with_shutdown(config: BusConfig, shutdown: Arc<AtomicBool>) -> Result<()> {
    let context = ZmqContext::new();
    let xsub = context
        .socket(SocketType::XSUB)
        .context("create XSUB socket")?;
    let xpub = context
        .socket(SocketType::XPUB)
        .context("create XPUB socket")?;
    let control = context
        .socket(SocketType::PAIR)
        .context("create proxy control PAIR")?;
    let control_client = context
        .socket(SocketType::PAIR)
        .context("create proxy control client")?;

    apply_low_latency_options(&xsub, config.snd_hwm, config.rcv_hwm)?;
    apply_low_latency_options(&xpub, config.snd_hwm, config.rcv_hwm)?;

    let xsub_endpoints = bind_all(&xsub, &config.xsub_bind, ports::XSUB_CHANNEL)?;
    let xpub_endpoints = bind_all(&xpub, &config.xpub_bind, ports::XPUB_CHANNEL)?;

    control
        .bind(PROXY_CONTROL)
        .context("bind proxy control")?;
    control_client
        .connect(PROXY_CONTROL)
        .context("connect proxy control")?;

    println!(
        "message_bus_broker proxy started\n  \
         publishers (PUB) connect ->\n    {}\n  \
         subscribers (SUB) connect ->\n    {}\n  \
         transports: tcp + inproc + ipc per socket\n  \
         forwarding: opaque multipart frames, no payload parsing",
        format_endpoints(&xsub_endpoints),
        format_endpoints(&xpub_endpoints),
    );

    let mut xsub = xsub;
    let mut xpub = xpub;
    let mut control = control;
    let proxy = thread::spawn(move || {
        let _ = zmq::proxy_steerable(&mut xsub, &mut xpub, &mut control);
    });

    while !shutdown.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }

    let _ = control_client.send(b"TERMINATE".as_ref(), 0);
    proxy
        .join()
        .map_err(|e| anyhow::anyhow!("message bus proxy thread: {e:?}"))?;
    Ok(())
}

/// Run a transparent ZeroMQ XSUB/XPUB proxy until interrupted.
pub fn run(config: BusConfig) -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    shutdown::install(shutdown.clone());
    run_with_shutdown(config, shutdown)
}

fn apply_low_latency_options(socket: &Socket, snd_hwm: i32, rcv_hwm: i32) -> Result<()> {
    socket.set_linger(0).context("set linger")?;
    socket.set_sndhwm(snd_hwm).context("set sndhwm")?;
    socket.set_rcvhwm(rcv_hwm).context("set rcvhwm")?;
    socket.set_immediate(true).context("set immediate")?;
    Ok(())
}
