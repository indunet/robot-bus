//! ROS 2–style [`Node`]: named participant, then attached to an executor.
//!
//! Typical flow (matches ROS 2):
//! 1. `let mut node = Node::new("pilot");`
//! 2. `executor.add_node(&mut node)?;`
//! 3. `let pub_ = node.create_publisher("/robot1/imu")?; pub_.publish(&bytes)?;`
//! 4. `executor.spin()?;`
//!
//! Topic / service / action names are used as given (pass full paths yourself).

use std::sync::{Arc, MutexGuard};
use std::time::Duration;

use prost::Message;

use crate::action_bus::{ActionClient as BusActionClient, ActionMessage};
use crate::errors::{BusError, Result};
use crate::message_bus::Publisher as BusPublisher;
use crate::runtime::callback_group::{CallbackGroup, CallbackGroupType};
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::executors::ExecutorHandle;
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
use crate::runtime::timers::{TimerCallback, TimerHandle};
use crate::service_bus::ServiceClient as BusServiceClient;
use crate::transports::{
    action_backend_endpoint, action_frontend_endpoint, message_xpub_endpoint,
    message_xsub_endpoint, service_backend_endpoint, service_frontend_endpoint,
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
    pub service_frontend: Option<String>,
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
            service_frontend: None,
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

    pub fn service_frontend_endpoint(&self) -> Result<String> {
        match &self.service_frontend {
            Some(ep) => Ok(ep.clone()),
            None => {
                service_frontend_endpoint(&self.host, &self.transport).map_err(BusError::Protocol)
            }
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

/// Topic-bound publisher returned by [`Node::create_publisher`] (ROS 2 style).
///
/// Shares one underlying bus PUB socket per node; each handle remembers its topic.
#[derive(Clone)]
pub struct TopicPublisher {
    inner: Arc<BusPublisher>,
    topic: String,
}

impl TopicPublisher {
    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn publish(&self, payload: &[u8]) -> Result<()> {
        self.inner.publish(&self.topic, payload)
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.inner.high_water_mark()
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.inner.set_high_water_mark(hwm)
    }
}

/// Service-bound client returned by [`Node::create_client`] (ROS 2 style).
pub struct NodeServiceClient {
    inner: BusServiceClient,
    service_name: String,
}

impl NodeServiceClient {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn call(&self, body: &[u8], timeout: Option<Duration>) -> Result<Vec<u8>> {
        self.inner
            .call(&self.service_name, body, None, timeout)
    }

    pub fn call_with_id(
        &self,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        self.inner
            .call(&self.service_name, body, request_id, timeout)
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.inner.high_water_mark()
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.inner.set_high_water_mark(hwm)
    }
}

/// Action-bound client returned by [`Node::create_action_client`] (ROS 2 style).
pub struct NodeActionClient {
    inner: BusActionClient,
    action_name: String,
}

impl NodeActionClient {
    pub fn action_name(&self) -> &str {
        &self.action_name
    }

    pub fn send_goal(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        self.inner
            .send_goal(&self.action_name, body, goal_id, timeout)
    }

    pub fn cancel(
        &self,
        goal_id: &str,
        body: &[u8],
        timeout: Option<Duration>,
    ) -> Result<ActionMessage> {
        self.inner
            .cancel(&self.action_name, goal_id, body, timeout)
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.inner.high_water_mark()
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.inner.set_high_water_mark(hwm)
    }
}

/// Named participant (simplified ROS 2 `Node`).
///
/// Create with [`Node::new`], then [`crate::runtime::SingleThreadedExecutor::add_node`]
/// (or the multi-threaded variant) before subscriptions / timers / services.
pub struct Node {
    name: String,
    options: NodeOptions,
    executor: Option<ExecutorHandle>,
    publisher: Option<Arc<BusPublisher>>,
    subscriber_connected: bool,
    default_callback_group: CallbackGroup,
}

impl Node {
    /// Create a node that is not yet attached to an executor.
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::default())
    }

    /// Create a node with explicit broker connection options.
    pub fn with_options(name: impl Into<String>, options: NodeOptions) -> Self {
        Self {
            name: name.into(),
            options,
            executor: None,
            publisher: None,
            subscriber_connected: false,
            default_callback_group: CallbackGroup::mutually_exclusive(),
        }
    }

    pub(crate) fn attach_executor(&mut self, handle: ExecutorHandle) -> Result<()> {
        if self.executor.is_some() {
            return Err(BusError::Protocol(
                "node is already added to an executor".into(),
            ));
        }
        self.executor = Some(handle);
        Ok(())
    }

    fn require_executor(&self) -> Result<&ExecutorHandle> {
        self.executor.as_ref().ok_or_else(|| {
            BusError::Protocol(
                "call executor.add_node(&mut node) before subscriptions/timers/services".into(),
            )
        })
    }

    fn lock_executor(&self) -> Result<MutexGuard<'_, Executor>> {
        self.require_executor()?.lock()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn options(&self) -> &NodeOptions {
        &self.options
    }

    /// Default mutually exclusive group (ROS 2 node default callback group).
    pub fn default_callback_group(&self) -> &CallbackGroup {
        &self.default_callback_group
    }

    /// Create a callback group (ROS 2 `create_callback_group`).
    pub fn create_callback_group(&self, kind: CallbackGroupType) -> CallbackGroup {
        CallbackGroup::new(kind)
    }

    /// Shared executor handle after [`add_node`](crate::runtime::SingleThreadedExecutor::add_node).
    pub fn executor_handle(&self) -> Option<&ExecutorHandle> {
        self.executor.as_ref()
    }

    /// Create a topic publisher (ROS 2 `create_publisher`).
    ///
    /// Returns a handle; call [`TopicPublisher::publish`] to send. Multiple
    /// publishers on the same node share one bus PUB socket.
    pub fn create_publisher(&mut self, topic: impl Into<String>) -> Result<TopicPublisher> {
        self.ensure_bus_publisher(None)?;
        Ok(TopicPublisher {
            inner: Arc::clone(self.publisher.as_ref().expect("publisher just ensured")),
            topic: topic.into(),
        })
    }

    /// Like [`create_publisher`](Self::create_publisher), setting HWM on first socket connect.
    pub fn create_publisher_with_hwm(
        &mut self,
        topic: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<TopicPublisher> {
        self.ensure_bus_publisher(Some(hwm))?;
        Ok(TopicPublisher {
            inner: Arc::clone(self.publisher.as_ref().expect("publisher just ensured")),
            topic: topic.into(),
        })
    }

    fn ensure_bus_publisher(&mut self, hwm: Option<HighWaterMark>) -> Result<()> {
        if let Some(pub_) = &self.publisher {
            if let Some(hwm) = hwm {
                pub_.set_high_water_mark(hwm)?;
            }
            return Ok(());
        }
        let hwm = match hwm {
            Some(h) => h,
            None => match &self.executor {
                Some(exec) => exec.lock()?.stream_hwm(),
                None => HighWaterMark::STREAM,
            },
        };
        let endpoint = self.options.message_xsub_endpoint()?;
        self.publisher = Some(Arc::new(BusPublisher::with_hwm(Some(&endpoint), hwm)?));
        Ok(())
    }

    /// Current shared publisher HWM, if any publisher was created.
    pub fn publisher_hwm(&self) -> Result<Option<HighWaterMark>> {
        match &self.publisher {
            Some(pub_) => Ok(Some(pub_.high_water_mark()?)),
            None => Ok(None),
        }
    }

    /// Update shared publisher HWM (error if no publisher created yet).
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
    /// Pass `callback_group` like ROS 2; `None` uses the node's default
    /// mutually exclusive group.
    pub fn create_subscription(
        &mut self,
        topic: &str,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<()> {
        self.ensure_subscriber()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?.subscribe(topic, callback, group)
    }

    /// Subscribe with a protobuf-typed callback. Decode failures are skipped.
    pub fn create_subscription_typed<M, F>(
        &mut self,
        topic: &str,
        callback: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<()>
    where
        M: Message + Default + 'static,
        F: Fn(&str, M) + Send + Sync + 'static,
    {
        self.ensure_subscriber()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?
            .subscribe_typed::<M, F>(topic, callback, group)
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
    ///
    /// `callback_group: None` → default mutually exclusive group.
    pub fn create_timer(
        &mut self,
        period: Duration,
        callback: TimerCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<TimerHandle> {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?
            .create_timer(period, callback, group)
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
        callback_group: Option<&CallbackGroup>,
    ) -> Result<()> {
        let endpoint = self.options.service_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?.register_service(
            service_name,
            handler,
            group,
            Some(&endpoint),
            identity,
        )
    }

    /// Create a service client (ROS 2 `create_client`).
    ///
    /// Returns a handle bound to `service_name`; call [`NodeServiceClient::call`] to invoke.
    pub fn create_client(
        &mut self,
        service_name: impl Into<String>,
    ) -> Result<NodeServiceClient> {
        self.create_client_with_hwm(service_name, self.client_rpc_hwm())
    }

    /// Like [`create_client`](Self::create_client), with an explicit HWM.
    pub fn create_client_with_hwm(
        &mut self,
        service_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeServiceClient> {
        let endpoint = self.options.service_frontend_endpoint()?;
        Ok(NodeServiceClient {
            inner: BusServiceClient::with_hwm(Some(&endpoint), hwm)?,
            service_name: service_name.into(),
        })
    }

    fn client_rpc_hwm(&self) -> HighWaterMark {
        match &self.executor {
            Some(exec) => exec.lock().map(|e| e.rpc_hwm()).unwrap_or(HighWaterMark::RPC),
            None => HighWaterMark::RPC,
        }
    }

    /// Register an action worker (ROS 2 `create_action_server` / action worker).
    pub fn create_action(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        identity: Option<&str>,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<()> {
        let endpoint = self.options.action_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?.register_action(
            action_name,
            handler,
            group,
            Some(&endpoint),
            identity,
        )
    }

    /// Create an action client (ROS 2 `create_action_client`).
    ///
    /// Returns a handle bound to `action_name`; call [`NodeActionClient::send_goal`].
    pub fn create_action_client(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClient> {
        self.create_action_client_with_hwm(action_name, self.client_action_hwm())
    }

    /// Like [`create_action_client`](Self::create_action_client), with an explicit HWM.
    pub fn create_action_client_with_hwm(
        &mut self,
        action_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeActionClient> {
        let endpoint = self.options.action_frontend_endpoint()?;
        Ok(NodeActionClient {
            inner: BusActionClient::with_hwm(Some(&endpoint), hwm)?,
            action_name: action_name.into(),
        })
    }

    fn client_action_hwm(&self) -> HighWaterMark {
        match &self.executor {
            Some(exec) => exec
                .lock()
                .map(|e| e.action_hwm())
                .unwrap_or(HighWaterMark::ACTION),
            None => HighWaterMark::ACTION,
        }
    }

    /// Connect the executor-owned action client used by callback-style [`send_goal`](Self::send_goal).
    pub fn connect_action_client(&mut self) -> Result<()> {
        let endpoint = self.options.action_frontend_endpoint()?;
        self.lock_executor()?
            .connect_action_client(Some(&endpoint))
    }

    /// Submit a goal via the executor (callback receives FEEDBACK / RESULT). Prefer
    /// [`create_action_client`](Self::create_action_client) for a ROS 2–style sync handle.
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
        self.require_executor()?.shutdown_handle()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.require_executor()?.shutdown()
    }

    /// Convenience: spin the attached executor (same as `executor.spin()`).
    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.require_executor()?.spin_once(timeout)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        self.require_executor()?.spin_some(timeout)
    }

    pub fn spin(&self) -> Result<()> {
        self.require_executor()?.spin()
    }

    pub fn start(&self) -> Result<()> {
        self.require_executor()?.start()
    }

    pub fn stop(&self) -> Result<()> {
        self.require_executor()?.stop()
    }

    pub fn wait(&self) -> Result<()> {
        self.require_executor()?.wait()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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
    fn add_node_then_create_publisher() {
        let mut node = Node::new("pilot");
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut node).unwrap();
        assert_eq!(node.name(), "pilot");
        let pub_ = node.create_publisher("/robot1/imu").unwrap();
        assert_eq!(pub_.topic(), "/robot1/imu");
    }

    #[test]
    fn subscription_requires_add_node() {
        let mut node = Node::new("pilot");
        let err = node
            .create_subscription("/imu", Arc::new(|_, _| {}), None)
            .unwrap_err();
        assert!(err.to_string().contains("add_node"));
    }

    #[test]
    fn create_callback_group_kinds() {
        let node = Node::new("pilot");
        let exclusive = node.create_callback_group(CallbackGroupType::MutuallyExclusive);
        let reentrant = node.create_callback_group(CallbackGroupType::Reentrant);
        assert_eq!(exclusive.kind(), CallbackGroupType::MutuallyExclusive);
        assert_eq!(reentrant.kind(), CallbackGroupType::Reentrant);
        assert_ne!(exclusive.id(), reentrant.id());
    }
}
