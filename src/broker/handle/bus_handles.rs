//! Background handles for the three ZMQ buses.

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use zmq::Context as ZmqContext;

use super::super::action_bus::{self, ActionBusConfig, ActionMetrics};
use super::super::message_bus::{self, BusConfig, MessageMetrics};
use super::super::service_bus::{self, ServiceBusConfig, ServiceMetrics};

pub(super) const STARTUP_SETTLE: Duration = Duration::from_millis(50);

pub(super) fn join_broker_thread(name: &str, handle: JoinHandle<Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|e| anyhow!("{name} thread panicked: {e:?}"))?
        .with_context(|| format!("{name} exited with error"))
}

/// Wait for a bus thread to publish its resolved TCP binds (`:0` → real ports).
///
/// On disconnect, join the thread so a bind `EADDRINUSE` (or panic) is not
/// flattened into a generic timeout error.
fn recv_bound_endpoints(
    name: &str,
    bound_rx: Receiver<(String, String)>,
    handle: JoinHandle<Result<()>>,
) -> Result<((String, String), JoinHandle<Result<()>>)> {
    match bound_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(pair) => Ok((pair, handle)),
        Err(RecvTimeoutError::Timeout) => Err(anyhow!(
            "{name} failed to report bound endpoints: timed out after 5s"
        )),
        Err(RecvTimeoutError::Disconnected) => {
            let detail = match handle.join() {
                Ok(Ok(())) => "thread exited without reporting binds".to_string(),
                Ok(Err(err)) => format!("{err:#}"),
                Err(panic) => format!("thread panicked: {panic:?}"),
            };
            Err(anyhow!("{name} failed to report bound endpoints: {detail}"))
        }
    }
}

/// Background handle for the message bus (XSUB/XPUB proxy).
pub struct MessageBusBroker {
    pub xsub_bind: String,
    pub xpub_bind: String,
    /// Per-topic counters updated by the proxy (shared with the console when enabled).
    pub metrics: Arc<MessageMetrics>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl MessageBusBroker {
    /// Bind and run the message bus on a background thread.
    ///
    /// Pass `Some(metrics)` only when the console (or another observer) needs
    /// topic counters — that enables libzmq capture. `None` keeps the forward
    /// path as a plain `proxy_steerable` with no capture overhead.
    pub(crate) fn start_with_zmq(
        zmq: ZmqContext,
        config: BusConfig,
        metrics: Option<Arc<MessageMetrics>>,
    ) -> Result<Self> {
        // Always keep an Arc for `self.metrics` (console may read zeros if unused).
        let stored = metrics.clone().unwrap_or_else(MessageMetrics::new);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let (bound_tx, bound_rx) = std::sync::mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            message_bus::run_with_shutdown_ctx_bound(
                zmq,
                config,
                shutdown_flag,
                metrics,
                Some(bound_tx),
            )
        });
        let ((xsub_bind, xpub_bind), handle) =
            recv_bound_endpoints("message bus", bound_rx, handle)?;
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            xsub_bind,
            xpub_bind,
            metrics: stored,
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
    pub metrics: Arc<ServiceMetrics>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ServiceBusBroker {
    /// Bind and run the service bus on a background thread.
    pub(crate) fn start_with_zmq(
        zmq: ZmqContext,
        config: ServiceBusConfig,
        metrics: Option<Arc<ServiceMetrics>>,
    ) -> Result<Self> {
        let stored = metrics.clone().unwrap_or_else(ServiceMetrics::new);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let (bound_tx, bound_rx) = std::sync::mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            service_bus::run_with_shutdown_ctx_bound(
                zmq,
                config,
                shutdown_flag,
                metrics,
                Some(bound_tx),
            )
        });
        let ((frontend_bind, backend_bind), handle) =
            recv_bound_endpoints("service bus", bound_rx, handle)?;
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            frontend_bind,
            backend_bind,
            metrics: stored,
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
    pub metrics: Arc<ActionMetrics>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ActionBusBroker {
    /// Bind and run the action bus on a background thread.
    pub(crate) fn start_with_zmq(
        zmq: ZmqContext,
        config: ActionBusConfig,
        metrics: Option<Arc<ActionMetrics>>,
    ) -> Result<Self> {
        let stored = metrics.clone().unwrap_or_else(ActionMetrics::new);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let (bound_tx, bound_rx) = std::sync::mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            action_bus::run_with_shutdown_ctx_bound(
                zmq,
                config,
                shutdown_flag,
                metrics,
                Some(bound_tx),
            )
        });
        let ((frontend_bind, backend_bind), handle) =
            recv_bound_endpoints("action bus", bound_rx, handle)?;
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            frontend_bind,
            backend_bind,
            metrics: stored,
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
