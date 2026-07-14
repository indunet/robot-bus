//! In-process broker handles: start a bus on a background thread and stop it later.
//!
//! Prefer these over the CLI binaries when embedding brokers in application code.
//! Each `start` uses [`run_with_shutdown`](super::service_bus::run_with_shutdown) and does
//! **not** install a process-wide Ctrl+C handler (unlike the blocking `run` helpers).

use anyhow::{anyhow, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::action_bus::{self, ActionBusConfig};
use super::message_bus::{self, BusConfig};
use super::service_bus::{self, ServiceBusConfig};

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
    pub fn start(config: BusConfig) -> Result<Self> {
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
    pub fn stop(mut self) -> Result<()> {
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
    pub fn start(config: ServiceBusConfig) -> Result<Self> {
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
    pub fn stop(mut self) -> Result<()> {
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
    pub fn start(config: ActionBusConfig) -> Result<Self> {
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
    pub fn stop(mut self) -> Result<()> {
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

/// Configuration for starting all three buses in one process.
#[derive(Clone, Debug, Default)]
pub struct RobotBusConfig {
    pub message: BusConfig,
    pub service: ServiceBusConfig,
    pub action: ActionBusConfig,
}

/// Background handle that owns message + service + action brokers.
pub struct RobotBusBroker {
    pub message: MessageBusBroker,
    pub service: ServiceBusBroker,
    pub action: ActionBusBroker,
}

impl RobotBusBroker {
    /// Start message, service, and action buses on background threads.
    pub fn start(config: RobotBusConfig) -> Result<Self> {
        let message = MessageBusBroker::start(config.message)?;
        let service = ServiceBusBroker::start(config.service)?;
        let action = ActionBusBroker::start(config.action)?;
        Ok(Self {
            message,
            service,
            action,
        })
    }

    /// Stop all buses and join their threads.
    pub fn stop(self) -> Result<()> {
        // Stop in reverse start order; collect first error.
        let action = self.action.stop();
        let service = self.service.stop();
        let message = self.message.stop();
        action.and(service).and(message)
    }
}
