mod bridge;
mod broker;
mod metrics;
mod peer;
mod ports;

pub use broker::{
    WorkerRegistry, build_client_reply, build_error_body, build_worker_cancel, build_worker_goal,
    run_loop,
};
pub use metrics::{ActionMetrics, ActionMetricsSnapshot, ActionSnapshot};
pub use peer::ActionPeer;
pub use ports::{
    BACKEND_PORT, DEFAULT_BACKEND_BIND, DEFAULT_FRONTEND_BIND, DEFAULT_HEARTBEAT_INTERVAL_MS,
    DEFAULT_HEARTBEAT_TIMEOUT_MS, DEFAULT_PENDING_TIMEOUT_MS, DEFAULT_RCV_HWM, DEFAULT_SND_HWM,
    FRONTEND_PORT,
};

use crate::shutdown;
use crate::transports::{BindAllOpts, bind_all, bind_tcp, format_endpoints};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::SyncSender;
use zmq::{Context as ZmqContext, Socket, SocketType};

#[derive(Clone, Debug)]
pub struct ActionBusConfig {
    pub frontend_bind: String,
    pub backend_bind: String,
    pub snd_hwm: i32,
    pub rcv_hwm: i32,
    pub heartbeat_interval_ms: u64,
    pub heartbeat_timeout_ms: u64,
    /// Give up on a queued goal (no worker appeared) after this many ms.
    pub pending_timeout_ms: u64,
    /// When true (default), also bind inproc + ipc aliases via [`bind_all`].
    /// Tests that spawn many brokers on ephemeral TCP ports should set this
    /// false so they do not fight over the fixed `/tmp/robot_bus/*.ipc` paths.
    pub bind_all_transports: bool,
    /// IPC / inproc namespace for [`bind_all`].
    pub bind_opts: BindAllOpts,
    /// Stable id for hop-path loop prevention. Empty → random UUID at start.
    pub broker_id: String,
    /// Static peers for action federation (empty → plain dual-ROUTER loop).
    pub peers: Vec<ActionPeer>,
}

impl Default for ActionBusConfig {
    fn default() -> Self {
        Self {
            frontend_bind: DEFAULT_FRONTEND_BIND.to_string(),
            backend_bind: DEFAULT_BACKEND_BIND.to_string(),
            snd_hwm: DEFAULT_SND_HWM,
            rcv_hwm: DEFAULT_RCV_HWM,
            heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
            heartbeat_timeout_ms: DEFAULT_HEARTBEAT_TIMEOUT_MS,
            pending_timeout_ms: DEFAULT_PENDING_TIMEOUT_MS,
            bind_all_transports: true,
            bind_opts: BindAllOpts::default(),
            broker_id: String::new(),
            peers: Vec::new(),
        }
    }
}

/// Run the dual-ROUTER action bus broker until `shutdown` is set.
pub fn run_with_shutdown(
    config: ActionBusConfig,
    shutdown: Arc<AtomicBool>,
    metrics: Option<Arc<ActionMetrics>>,
) -> Result<()> {
    run_with_shutdown_ctx(ZmqContext::new(), config, shutdown, metrics)
}

/// Like [`run_with_shutdown`], but sockets are created from the given context
/// (required for same-process `inproc://` with SDK participants).
pub fn run_with_shutdown_ctx(
    context: ZmqContext,
    config: ActionBusConfig,
    shutdown: Arc<AtomicBool>,
    metrics: Option<Arc<ActionMetrics>>,
) -> Result<()> {
    run_with_shutdown_ctx_bound(context, config, shutdown, metrics, None)
}

/// Like [`run_with_shutdown_ctx`], optionally reporting resolved TCP binds.
pub fn run_with_shutdown_ctx_bound(
    context: ZmqContext,
    config: ActionBusConfig,
    shutdown: Arc<AtomicBool>,
    metrics: Option<Arc<ActionMetrics>>,
    bound_tx: Option<SyncSender<(String, String)>>,
) -> Result<()> {
    let frontend = context
        .socket(SocketType::ROUTER)
        .context("create frontend ROUTER")?;
    let backend = context
        .socket(SocketType::ROUTER)
        .context("create backend ROUTER")?;

    apply_low_latency_options(&frontend, config.snd_hwm, config.rcv_hwm)?;
    apply_low_latency_options(&backend, config.snd_hwm, config.rcv_hwm)?;

    let (resolved_fe, frontend_endpoints, resolved_be, backend_endpoints) =
        if config.bind_all_transports {
            let (rfe, fe_eps) = bind_all(
                &frontend,
                &config.frontend_bind,
                ports::FRONTEND_CHANNEL,
                &config.bind_opts,
            )?;
            let (rbe, be_eps) = bind_all(
                &backend,
                &config.backend_bind,
                ports::BACKEND_CHANNEL,
                &config.bind_opts,
            )?;
            (rfe, fe_eps, rbe, be_eps)
        } else {
            let rfe = bind_tcp(&frontend, &config.frontend_bind)?;
            let rbe = bind_tcp(&backend, &config.backend_bind)?;
            (rfe.clone(), vec![rfe], rbe.clone(), vec![rbe])
        };

    if let Some(tx) = bound_tx {
        let _ = tx.send((resolved_fe, resolved_be));
    }

    println!(
        "action_bus_broker broker started\n  \
         clients (DEALER) connect ->\n    {}\n  \
         workers (DEALER) connect ->\n    {}\n  \
         transports: {}\n  \
         routing: by action_name frame, goal_id tracked, body opaque{}",
        format_endpoints(&frontend_endpoints),
        format_endpoints(&backend_endpoints),
        if config.bind_all_transports {
            "tcp + inproc + ipc per socket"
        } else {
            "tcp only"
        },
        if config.peers.is_empty() {
            String::new()
        } else {
            format!("\n  federation: {} peer(s)", config.peers.len())
        },
    );

    if config.peers.is_empty() {
        broker::run_loop(&frontend, &backend, &config, &shutdown, metrics.as_ref())
            .context("broker loop")?;
    } else {
        bridge::run_federated(
            &context,
            frontend,
            backend,
            &config,
            &shutdown,
            metrics.as_ref(),
        )
        .context("federated action broker loop")?;
    }
    Ok(())
}

/// Run the dual-ROUTER action bus broker until interrupted.
pub fn run(config: ActionBusConfig) -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    shutdown::install(shutdown.clone());
    run_with_shutdown(config, shutdown, None)
}

fn apply_low_latency_options(socket: &Socket, snd_hwm: i32, rcv_hwm: i32) -> Result<()> {
    socket.set_linger(0).context("set linger")?;
    socket.set_sndhwm(snd_hwm).context("set sndhwm")?;
    socket.set_rcvhwm(rcv_hwm).context("set rcvhwm")?;
    socket.set_immediate(true).context("set immediate")?;
    Ok(())
}
