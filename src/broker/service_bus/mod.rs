mod broker;
mod ports;

pub use broker::{
    build_client_reply, build_error_body, build_worker_forward, parse_service_name, run_loop,
    WorkerRegistry,
};
pub use ports::{
    DEFAULT_BACKEND_BIND, DEFAULT_FRONTEND_BIND, DEFAULT_HEARTBEAT_INTERVAL_MS,
    DEFAULT_HEARTBEAT_TIMEOUT_MS, DEFAULT_RCV_HWM, DEFAULT_SND_HWM, BACKEND_PORT, FRONTEND_PORT,
};

use anyhow::{Context, Result};
use crate::shutdown;
use crate::transports::{bind_all, format_endpoints};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use zmq::{Context as ZmqContext, Socket, SocketType};

#[derive(Clone, Debug)]
pub struct ServiceBusConfig {
    pub frontend_bind: String,
    pub backend_bind: String,
    pub snd_hwm: i32,
    pub rcv_hwm: i32,
    pub heartbeat_interval_ms: u64,
    pub heartbeat_timeout_ms: u64,
}

impl Default for ServiceBusConfig {
    fn default() -> Self {
        Self {
            frontend_bind: DEFAULT_FRONTEND_BIND.to_string(),
            backend_bind: DEFAULT_BACKEND_BIND.to_string(),
            snd_hwm: DEFAULT_SND_HWM,
            rcv_hwm: DEFAULT_RCV_HWM,
            heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
            heartbeat_timeout_ms: DEFAULT_HEARTBEAT_TIMEOUT_MS,
        }
    }
}

/// Run the dual-ROUTER service bus broker until `shutdown` is set.
pub fn run_with_shutdown(config: ServiceBusConfig, shutdown: Arc<AtomicBool>) -> Result<()> {
    let context = ZmqContext::new();
    let frontend = context
        .socket(SocketType::ROUTER)
        .context("create frontend ROUTER")?;
    let backend = context
        .socket(SocketType::ROUTER)
        .context("create backend ROUTER")?;

    apply_low_latency_options(&frontend, config.snd_hwm, config.rcv_hwm)?;
    apply_low_latency_options(&backend, config.snd_hwm, config.rcv_hwm)?;

    let frontend_endpoints = bind_all(&frontend, &config.frontend_bind, ports::FRONTEND_CHANNEL)?;
    let backend_endpoints = bind_all(&backend, &config.backend_bind, ports::BACKEND_CHANNEL)?;

    println!(
        "service_bus_broker broker started\n  \
         clients (REQ) connect ->\n    {}\n  \
         workers (DEALER) connect ->\n    {}\n  \
         transports: tcp + inproc + ipc per socket\n  \
         routing: by service_name frame, body opaque",
        format_endpoints(&frontend_endpoints),
        format_endpoints(&backend_endpoints),
    );

    broker::run_loop(&frontend, &backend, &config, &shutdown).context("broker loop")?;
    Ok(())
}

/// Run the dual-ROUTER service bus broker until interrupted.
pub fn run(config: ServiceBusConfig) -> Result<()> {
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
