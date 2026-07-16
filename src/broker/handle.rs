//! In-process broker handle: start all buses (and gRPC) on background threads.
//!
//! Prefer [`RobotBusBroker::start`] over the CLI binary when embedding in application code.
//! `start` uses per-bus [`run_with_shutdown`](super::service_bus::run_with_shutdown) and does
//! **not** install a process-wide Ctrl+C handler (unlike the blocking `run` helpers).

use anyhow::{anyhow, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::action_bus::{self, ActionBusConfig};
use super::message_bus::{self, BusConfig};
use super::service_bus::{self, ServiceBusConfig};

#[cfg(feature = "grpc")]
use crate::grpc::{serve_with_shutdown, GatewayConfig};
#[cfg(feature = "grpc")]
use std::net::SocketAddr;

const STARTUP_SETTLE: Duration = Duration::from_millis(50);

fn join_broker_thread(name: &str, handle: JoinHandle<Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|e| anyhow!("{name} thread panicked: {e:?}"))?
        .with_context(|| format!("{name} exited with error"))
}

/// Background handle for the message bus (XSUB/XPUB proxy).
pub struct MessageBusBroker {
    pub xsub_bind: String,
    pub xpub_bind: String,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl MessageBusBroker {
    /// Bind and run the message bus on a background thread.
    pub(crate) fn start(config: BusConfig) -> Result<Self> {
        let xsub_bind = config.xsub_bind.clone();
        let xpub_bind = config.xpub_bind.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let handle = thread::spawn(move || message_bus::run_with_shutdown(config, shutdown_flag));
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            xsub_bind,
            xpub_bind,
            shutdown,
            handle: Some(handle),
        })
    }

    /// Signal shutdown and join the broker thread.
    pub(crate) fn stop(mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            join_broker_thread("message_bus", handle)?;
        }
        Ok(())
    }
}

impl Drop for MessageBusBroker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Background handle for the service bus (dual-ROUTER).
pub struct ServiceBusBroker {
    pub frontend_bind: String,
    pub backend_bind: String,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ServiceBusBroker {
    /// Bind and run the service bus on a background thread.
    pub(crate) fn start(config: ServiceBusConfig) -> Result<Self> {
        let frontend_bind = config.frontend_bind.clone();
        let backend_bind = config.backend_bind.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let handle = thread::spawn(move || service_bus::run_with_shutdown(config, shutdown_flag));
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            frontend_bind,
            backend_bind,
            shutdown,
            handle: Some(handle),
        })
    }

    /// Signal shutdown and join the broker thread.
    pub(crate) fn stop(mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            join_broker_thread("service_bus", handle)?;
        }
        Ok(())
    }
}

impl Drop for ServiceBusBroker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Background handle for the action bus (dual-ROUTER).
pub struct ActionBusBroker {
    pub frontend_bind: String,
    pub backend_bind: String,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ActionBusBroker {
    /// Bind and run the action bus on a background thread.
    pub(crate) fn start(config: ActionBusConfig) -> Result<Self> {
        let frontend_bind = config.frontend_bind.clone();
        let backend_bind = config.backend_bind.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let handle = thread::spawn(move || action_bus::run_with_shutdown(config, shutdown_flag));
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            frontend_bind,
            backend_bind,
            shutdown,
            handle: Some(handle),
        })
    }

    /// Signal shutdown and join the broker thread.
    pub(crate) fn stop(mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            join_broker_thread("action_bus", handle)?;
        }
        Ok(())
    }
}

impl Drop for ActionBusBroker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// gRPC / gRPC-Web listen options (feature `grpc`, enabled by default).
#[cfg(feature = "grpc")]
#[derive(Clone, Debug)]
pub struct GrpcBrokerConfig {
    pub listen: SocketAddr,
    /// When empty, allow any origin (local-dev default).
    pub cors_origins: Vec<String>,
}

#[cfg(feature = "grpc")]
impl Default for GrpcBrokerConfig {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:15770".parse().expect("default grpc listen"),
            cors_origins: Vec::new(),
        }
    }
}

/// Configuration for starting all buses (and gRPC) in one process.
#[derive(Clone, Debug, Default)]
pub struct RobotBusConfig {
    pub message: BusConfig,
    pub service: ServiceBusConfig,
    pub action: ActionBusConfig,
    #[cfg(feature = "grpc")]
    pub grpc: GrpcBrokerConfig,
}

#[cfg(feature = "grpc")]
struct GrpcGatewayHandle {
    pub listen: SocketAddr,
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    handle: Option<JoinHandle<Result<()>>>,
}

#[cfg(feature = "grpc")]
impl GrpcGatewayHandle {
    fn start(config: GatewayConfig) -> Result<Self> {
        let listen = config.listen;
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("create tokio runtime for gRPC gateway")?;
            rt.block_on(async move {
                let mut graceful_rx = shutdown_rx.clone();
                let graceful = async move {
                    while !*graceful_rx.borrow() {
                        if graceful_rx.changed().await.is_err() {
                            break;
                        }
                    }
                };
                let mut force_rx = shutdown_rx;
                let force = async move {
                    while !*force_rx.borrow() {
                        if force_rx.changed().await.is_err() {
                            break;
                        }
                    }
                    // Open client streams can block tonic's graceful drain forever;
                    // after a short grace period, drop the server future and free the port.
                    tokio::time::sleep(Duration::from_millis(500)).await;
                };
                tokio::select! {
                    result = serve_with_shutdown(config, graceful) => result,
                    _ = force => Ok(()),
                }
            })
        });
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            listen,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        })
    }

    fn stop(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            join_broker_thread("grpc_gateway", handle)?;
        }
        Ok(())
    }
}

#[cfg(feature = "grpc")]
impl Drop for GrpcGatewayHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Rewrite a bind address into a client connect endpoint (`0.0.0.0` / `*` → `127.0.0.1`).
fn bind_to_connect(bind: &str) -> String {
    if let Some(rest) = bind.strip_prefix("tcp://0.0.0.0:") {
        return format!("tcp://127.0.0.1:{rest}");
    }
    if let Some(rest) = bind.strip_prefix("tcp://*:") {
        return format!("tcp://127.0.0.1:{rest}");
    }
    bind.to_string()
}

/// Background handle that owns message + service + action brokers (and gRPC when enabled).
pub struct RobotBusBroker {
    pub message: MessageBusBroker,
    pub service: ServiceBusBroker,
    pub action: ActionBusBroker,
    #[cfg(feature = "grpc")]
    grpc: GrpcGatewayHandle,
}

impl RobotBusBroker {
    /// Start message, service, and action buses (and the gRPC gateway) on background threads.
    pub fn start(config: RobotBusConfig) -> Result<Self> {
        let message = MessageBusBroker::start(config.message)?;
        let service = ServiceBusBroker::start(config.service)?;
        let action = ActionBusBroker::start(config.action)?;

        #[cfg(feature = "grpc")]
        let grpc = {
            let gateway = GatewayConfig {
                listen: config.grpc.listen,
                message_xpub: bind_to_connect(&message.xpub_bind),
                service_frontend: bind_to_connect(&service.frontend_bind),
                action_frontend: bind_to_connect(&action.frontend_bind),
                cors_origins: config.grpc.cors_origins,
            };
            GrpcGatewayHandle::start(gateway)?
        };

        Ok(Self {
            message,
            service,
            action,
            #[cfg(feature = "grpc")]
            grpc,
        })
    }

    /// gRPC / gRPC-Web listen address (feature `grpc`).
    #[cfg(feature = "grpc")]
    pub fn grpc_listen(&self) -> SocketAddr {
        self.grpc.listen
    }

    /// Stop all buses (and gRPC) and join their threads.
    pub fn stop(self) -> Result<()> {
        // Stop in reverse start order; collect first error.
        #[cfg(feature = "grpc")]
        let grpc = self.grpc.stop();
        let action = self.action.stop();
        let service = self.service.stop();
        let message = self.message.stop();
        #[cfg(feature = "grpc")]
        {
            return grpc.and(action).and(service).and(message);
        }
        #[cfg(not(feature = "grpc"))]
        {
            action.and(service).and(message)
        }
    }
}
