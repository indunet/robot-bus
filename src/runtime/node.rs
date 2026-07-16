//! ROS 2–style [`Node`]: named participant attached to an executor.
//!
//! Owns broker connection config ([`NodeOptions`]) plus a node name.
//! Topic / service / action names are used as given (pass full paths yourself).
//! Create via [`crate::runtime::SingleThreadedExecutor::create_node`] or
//! [`crate::runtime::MultiThreadedExecutor::create_node`], then `spin` the executor.

use std::sync::MutexGuard;
use std::time::Duration;

use prost::Message;

use crate::errors::{BusError, Result};
use crate::message_bus::Publisher;
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::executors::ExecutorHandle;
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
use crate::runtime::timers::{TimerCallback, TimerHandle};
use crate::transports::{
    action_backend_endpoint, action_frontend_endpoint, message_xpub_endpoint,
    message_xsub_endpoint, service_backend_endpoint,
};
use crate::zmq_helpers::HighWaterMark;

/// Broker connection settings owned by a [`Node`].
///
/// Defaults: `host = "localhost"`, `transport = "tcp"`. Explicit endpoint
/// fields override the derived `transports::*` addresses when set.
#[derive(Debug, Clone)]
pub struct NodeOptions {
    pub host: String,
    pub transport: String,
    pub message_xsub: Option<String>,
    pub message_xpub: Option<String>,
    pub service_backend: Option<String>,
    pub action_backend: Option<String>,
    pub action_frontend: Option<String>,
}

impl Default for NodeOptions {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            transport: "tcp".into(),
            message_xsub: None,
            message_xpub: None,
            service_backend: None,
            action_backend: None,
            action_frontend: None,
        }
    }
}

impl NodeOptions {
    pub fn message_xsub_endpoint(&self) -> Result<String> {
        match &self.message_xsub {
            Some(ep) => Ok(ep.clone()),
            None => message_xsub_endpoint(&self.host, &self.transport).map_err(BusError::Protocol),
        }
    }

    pub fn message_xpub_endpoint(&self) -> Result<String> {
        match &self.message_xpub {
            Some(ep) => Ok(ep.clone()),
            None => message_xpub_endpoint(&self.host, &self.transport).map_err(BusError::Protocol),
        }
    }

    pub fn service_backend_endpoint(&self) -> Result<String> {
        match &self.service_backend {
            Some(ep) => Ok(ep.clone()),
            None => {
                service_backend_endpoint(&self.host, &self.transport).map_err(BusError::Protocol)
            }
        }
    }

    pub fn action_backend_endpoint(&self) -> Result<String> {
        match &self.action_backend {
            Some(ep) => Ok(ep.clone()),
            None => action_backend_endpoint(&self.host, &self.transport).map_err(BusError::Protocol),
        }
    }

    pub fn action_frontend_endpoint(&self) -> Result<String> {
        match &self.action_frontend {
            Some(ep) => Ok(ep.clone()),
            None => {
                action_frontend_endpoint(&self.host, &self.transport).map_err(BusError::Protocol)
            }
        }
    }
}

/// Named participant attached to a shared executor (simplified ROS 2 `Node`).
pub struct Node {
    name: String,
    options: NodeOptions,
    executor: ExecutorHandle,
    publisher: Option<Publisher>,
    subscriber_connected: bool,
}

impl Node {
    pub(crate) fn attach(
        name: impl Into<String>,
        options: NodeOptions,
        executor: ExecutorHandle,
    ) -> Self {
        Self {
            name: name.into(),
            options,
            executor,
            publisher: None,
            subscriber_connected: false,
        }
    }

    fn lock_executor(&self) -> Result<MutexGuard<'_, Executor>> {
        self.executor.lock()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn options(&self) -> &NodeOptions {
        &self.options
    }

    /// Shared executor handle (same as the parent `*ThreadedExecutor`).
    pub fn executor_handle(&self) -> &ExecutorHandle {
        &self.executor
    }

    /// Connect a publisher using this node's message XSUB endpoint.
    ///
    /// Call once; subsequent calls replace the publisher. Uses stream HWM defaults.
    pub fn create_publisher(&mut self) -> Result<()> {
        let hwm = self.lock_executor()?.stream_hwm();
        self.create_publisher_with_hwm(hwm)
    }

    /// Connect a publisher with an explicit high-water mark.
    pub fn create_publisher_with_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        let endpoint = self.options.message_xsub_endpoint()?;
        self.publisher = Some(Publisher::with_hwm(Some(&endpoint), hwm)?);
        Ok(())
    }

    /// Publish on `topic` (use a full path). Requires [`create_publisher`].
    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let Some(pub_) = self.publisher.as_ref() else {
            return Err(BusError::Protocol(
                "create_publisher() before publish()".into(),
            ));
        };
        pub_.publish(topic, payload)
    }

    /// Current publisher HWM, if a publisher exists.
    pub fn publisher_hwm(&self) -> Result<Option<HighWaterMark>> {
        match &self.publisher {
            Some(pub_) => Ok(Some(pub_.high_water_mark()?)),
            None => Ok(None),
        }
    }

    /// Update publisher HWM (error if publisher not created).
    pub fn set_publisher_hwm(&self, hwm: HighWaterMark) -> Result<()> {
        let Some(pub_) = self.publisher.as_ref() else {
            return Err(BusError::Protocol(
                "create_publisher() before set_publisher_hwm()".into(),
            ));
        };
        pub_.set_high_water_mark(hwm)
    }

    pub fn stream_hwm(&self) -> Result<HighWaterMark> {
        Ok(self.lock_executor()?.stream_hwm())
    }

    pub fn set_stream_hwm(&self, hwm: HighWaterMark) -> Result<()> {
        self.lock_executor()?.set_stream_hwm(hwm)
    }

    pub fn rpc_hwm(&self) -> Result<HighWaterMark> {
        Ok(self.lock_executor()?.rpc_hwm())
    }

    pub fn set_rpc_hwm(&self, hwm: HighWaterMark) -> Result<()> {
        self.lock_executor()?.set_rpc_hwm(hwm)
    }

    pub fn action_hwm(&self) -> Result<HighWaterMark> {
        Ok(self.lock_executor()?.action_hwm())
    }

    pub fn set_action_hwm(&self, hwm: HighWaterMark) -> Result<()> {
        self.lock_executor()?.set_action_hwm(hwm)
    }

    /// Subscribe with a raw-bytes callback (ROS 2 `create_subscription`).
    ///
    /// Connects the subscriber on first use using the node's message XPUB endpoint.
    pub fn create_subscription(
        &mut self,
        topic: &str,
        callback: MessageCallback,
    ) -> Result<()> {
        self.ensure_subscriber()?;
        self.lock_executor()?.subscribe(topic, callback)
    }

    /// Subscribe with a protobuf-typed callback. Decode failures are skipped.
    pub fn create_subscription_typed<M, F>(&mut self, topic: &str, callback: F) -> Result<()>
    where
        M: Message + Default + 'static,
        F: Fn(&str, M) + Send + Sync + 'static,
    {
        self.ensure_subscriber()?;
        self.lock_executor()?
            .subscribe_typed::<M, F>(topic, callback)
    }

    fn ensure_subscriber(&mut self) -> Result<()> {
        if !self.subscriber_connected {
            let endpoint = self.options.message_xpub_endpoint()?;
            self.lock_executor()?
                .connect_subscriber(Some(&endpoint))?;
            self.subscriber_connected = true;
        }
        Ok(())
    }

    /// Periodic timer (ROS 2 `create_timer`).
    pub fn create_timer(
        &mut self,
        period: Duration,
        callback: TimerCallback,
    ) -> Result<TimerHandle> {
        self.lock_executor()?.create_timer(period, callback)
    }

    pub fn cancel_timer(&mut self, handle: TimerHandle) -> Result<()> {
        self.lock_executor()?.cancel_timer(handle)
    }

    /// Register a service worker (ROS 2 `create_service`).
    pub fn create_service(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        identity: Option<&str>,
    ) -> Result<()> {
        let endpoint = self.options.service_backend_endpoint()?;
        self.lock_executor()?
            .register_service(service_name, handler, Some(&endpoint), identity)
    }

    /// Register an action worker (ROS 2 `create_action_server` / action worker).
    pub fn create_action(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        identity: Option<&str>,
    ) -> Result<()> {
        let endpoint = self.options.action_backend_endpoint()?;
        self.lock_executor()?
            .register_action(action_name, handler, Some(&endpoint), identity)
    }

    pub fn connect_action_client(&mut self) -> Result<()> {
        let endpoint = self.options.action_frontend_endpoint()?;
        self.lock_executor()?
            .connect_action_client(Some(&endpoint))
    }

    pub fn send_goal(
        &self,
        action_name: &str,
        body: &[u8],
        callback: ActionMessageCallback,
        goal_id: Option<&str>,
    ) -> Result<String> {
        self.lock_executor()?
            .send_goal(action_name, body, callback, goal_id)
    }

    pub fn cancel_goal(&self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        self.lock_executor()?
            .cancel_goal(action_name, goal_id, body)
    }

    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        self.executor.shutdown_handle()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.executor.shutdown()
    }

    /// Convenience: spin the shared executor (same as `executor.spin()`).
    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.executor.spin_once(timeout)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        self.executor.spin_some(timeout)
    }

    pub fn spin(&self) -> Result<()> {
        self.executor.spin()
    }

    pub fn start(&self) -> Result<()> {
        self.executor.start()
    }

    pub fn stop(&self) -> Result<()> {
        self.executor.stop()
    }

    pub fn wait(&self) -> Result<()> {
        self.executor.wait()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::SingleThreadedExecutor;

    #[test]
    fn options_override_endpoints() {
        let opts = NodeOptions {
            message_xpub: Some("tcp://127.0.0.1:9999".into()),
            ..NodeOptions::default()
        };
        assert_eq!(
            opts.message_xpub_endpoint().unwrap(),
            "tcp://127.0.0.1:9999"
        );
    }

    #[test]
    fn name_is_stored() {
        let executor = SingleThreadedExecutor::new();
        let node = executor.create_node("pilot");
        assert_eq!(node.name(), "pilot");
    }
}
