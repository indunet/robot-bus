//! ROS 2–style [`Node`]: named facade over [`super::BusRuntime`].
//!
//! First version keeps it thin: node name / namespace remapping plus
//! `create_subscription` / `create_publisher` / `create_timer` / … sugar.
//! The executor is still [`BusRuntime`] (`spin` / `spin_once` / `spin_some`).

use std::time::Duration;

use crate::errors::{BusError, Result};
use crate::message_bus::Publisher;
use crate::runtime::bus_runtime::{BusRuntime, ShutdownHandle};
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
use crate::runtime::timers::{TimerCallback, TimerHandle};
use crate::zmq_helpers::HighWaterMark;

/// Named participant that owns a [`BusRuntime`] (simplified ROS 2 `Node`).
pub struct Node {
    name: String,
    namespace: String,
    runtime: BusRuntime,
    publisher: Option<Publisher>,
    subscriber_connected: bool,
}

impl Node {
    /// Create a node with an empty namespace (topics/services used as given).
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_options(name.into(), String::new(), BusRuntime::new())
    }

    /// Create a node under `namespace` (relative names are prefixed).
    pub fn with_namespace(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self::with_options(name.into(), namespace.into(), BusRuntime::new())
    }

    /// Like [`new`](Self::new), but service/action handlers may use a worker pool.
    pub fn with_executor(name: impl Into<String>, max_workers: usize) -> Self {
        Self::with_options(
            name.into(),
            String::new(),
            BusRuntime::with_executor(max_workers),
        )
    }

    /// Namespace + bounded worker pool.
    pub fn with_namespace_and_executor(
        name: impl Into<String>,
        namespace: impl Into<String>,
        max_workers: usize,
    ) -> Self {
        Self::with_options(
            name.into(),
            namespace.into(),
            BusRuntime::with_executor(max_workers),
        )
    }

    fn with_options(name: String, namespace: String, runtime: BusRuntime) -> Self {
        Self {
            name,
            namespace: normalize_namespace(namespace),
            runtime,
            publisher: None,
            subscriber_connected: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Fully qualified name: `/ns/name` or `/name` when namespace is empty.
    pub fn fully_qualified_name(&self) -> String {
        if self.namespace.is_empty() {
            format!("/{}", self.name.trim_matches('/'))
        } else {
            format!(
                "/{}/{}",
                self.namespace.trim_matches('/'),
                self.name.trim_matches('/')
            )
        }
    }

    /// Resolve a relative topic/service/action name against this node's namespace.
    ///
    /// - Names starting with `/` are absolute (leading `/` is kept on the wire).
    /// - Otherwise: `{namespace}/{name}` when namespace is set, else `name` as-is.
    pub fn resolve_name(&self, name: &str) -> String {
        resolve_name(&self.namespace, name)
    }

    /// Connect a publisher (ROS 2 `create_publisher` without typed msgs).
    ///
    /// Call once; subsequent calls replace the publisher. Uses stream HWM defaults.
    pub fn create_publisher(&mut self, endpoint: Option<&str>) -> Result<()> {
        self.create_publisher_with_hwm(endpoint, self.runtime.stream_hwm())
    }

    /// Connect a publisher with an explicit high-water mark.
    pub fn create_publisher_with_hwm(
        &mut self,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<()> {
        self.publisher = Some(Publisher::with_hwm(endpoint, hwm)?);
        Ok(())
    }

    /// Publish on a (possibly namespaced) topic. Requires [`create_publisher`].
    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let Some(pub_) = self.publisher.as_ref() else {
            return Err(BusError::Protocol(
                "create_publisher() before publish()".into(),
            ));
        };
        pub_.publish(&self.resolve_name(topic), payload)
    }

    /// Current publisher HWM, if a publisher exists.
    pub fn publisher_hwm(&self) -> Result<Option<HighWaterMark>> {
        match &self.publisher {
            Some(pub_) => Ok(Some(pub_.high_water_mark()?)),
            None => Ok(None),
        }
    }

    /// Update publisher HWM (no-op error if publisher not created).
    pub fn set_publisher_hwm(&self, hwm: HighWaterMark) -> Result<()> {
        let Some(pub_) = self.publisher.as_ref() else {
            return Err(BusError::Protocol(
                "create_publisher() before set_publisher_hwm()".into(),
            ));
        };
        pub_.set_high_water_mark(hwm)
    }

    pub fn stream_hwm(&self) -> HighWaterMark {
        self.runtime.stream_hwm()
    }

    pub fn set_stream_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        self.runtime.set_stream_hwm(hwm)
    }

    pub fn rpc_hwm(&self) -> HighWaterMark {
        self.runtime.rpc_hwm()
    }

    pub fn set_rpc_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        self.runtime.set_rpc_hwm(hwm)
    }

    pub fn action_hwm(&self) -> HighWaterMark {
        self.runtime.action_hwm()
    }

    pub fn set_action_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        self.runtime.set_action_hwm(hwm)
    }

    /// Subscribe with a callback (ROS 2 `create_subscription`).
    ///
    /// Connects the subscriber on first use. `endpoint` is only used when
    /// connecting; later calls ignore it.
    pub fn create_subscription(
        &mut self,
        topic: &str,
        callback: MessageCallback,
        endpoint: Option<&str>,
    ) -> Result<()> {
        if !self.subscriber_connected {
            self.runtime.connect_subscriber(endpoint)?;
            self.subscriber_connected = true;
        }
        self.runtime.subscribe(&self.resolve_name(topic), callback)
    }

    /// Periodic timer (ROS 2 `create_timer`).
    pub fn create_timer(
        &mut self,
        period: Duration,
        callback: TimerCallback,
    ) -> Result<TimerHandle> {
        self.runtime.create_timer(period, callback)
    }

    pub fn cancel_timer(&mut self, handle: TimerHandle) -> Result<()> {
        self.runtime.cancel_timer(handle)
    }

    /// Register a service worker (ROS 2 `create_service`).
    pub fn create_service(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        backend_endpoint: Option<&str>,
        identity: Option<&str>,
    ) -> Result<()> {
        self.runtime.register_service(
            &self.resolve_name(service_name),
            handler,
            backend_endpoint,
            identity,
        )
    }

    /// Register an action worker (ROS 2 `create_action_server` / action worker).
    pub fn create_action(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        backend_endpoint: Option<&str>,
        identity: Option<&str>,
    ) -> Result<()> {
        self.runtime.register_action(
            &self.resolve_name(action_name),
            handler,
            backend_endpoint,
            identity,
        )
    }

    pub fn connect_action_client(&mut self, endpoint: Option<&str>) -> Result<()> {
        self.runtime.connect_action_client(endpoint)
    }

    pub fn send_goal(
        &self,
        action_name: &str,
        body: &[u8],
        callback: ActionMessageCallback,
        goal_id: Option<&str>,
    ) -> Result<String> {
        self.runtime
            .send_goal(&self.resolve_name(action_name), body, callback, goal_id)
    }

    pub fn cancel_goal(&self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        self.runtime
            .cancel_goal(&self.resolve_name(action_name), goal_id, body)
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.runtime.shutdown_handle()
    }

    pub fn shutdown(&self) {
        self.runtime.shutdown();
    }

    pub fn spin_once(&mut self, timeout: Option<Duration>) -> Result<bool> {
        self.runtime.spin_once(timeout)
    }

    pub fn spin_some(&mut self, timeout: Option<Duration>) -> Result<()> {
        self.runtime.spin_some(timeout)
    }

    pub fn spin(&mut self) -> Result<()> {
        self.runtime.spin()
    }

    pub fn start(&mut self) -> Result<()> {
        self.runtime.start()
    }

    pub fn stop(&mut self) {
        self.runtime.stop();
    }

    pub fn wait(&mut self) {
        self.runtime.wait();
    }

    /// Escape hatch to the underlying executor.
    pub fn runtime(&self) -> &BusRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut BusRuntime {
        &mut self.runtime
    }
}

fn normalize_namespace(namespace: String) -> String {
    namespace.trim().trim_matches('/').to_string()
}

fn resolve_name(namespace: &str, name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return name.to_string();
    }
    if name.starts_with('/') {
        return name.to_string();
    }
    if namespace.is_empty() {
        return name.to_string();
    }
    format!("{namespace}/{name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_relative_and_absolute() {
        let node = Node::with_namespace("pilot", "robot1");
        assert_eq!(node.resolve_name("imu"), "robot1/imu");
        assert_eq!(node.resolve_name("/absolute/topic"), "/absolute/topic");
        assert_eq!(node.fully_qualified_name(), "/robot1/pilot");
    }

    #[test]
    fn empty_namespace_keeps_name() {
        let node = Node::new("sensor");
        assert_eq!(node.resolve_name("wireless.imu"), "wireless.imu");
        assert_eq!(node.fully_qualified_name(), "/sensor");
    }
}
