//! ROS 2–style [`Node`]: named participant with optional executor.
//!
//! Recommended (ROS-like) flow:
//! 1. `let ctx = Context::new();`
//! 2. `let mut node = Node::with_context(&ctx, "pilot");`
//! 3. `node.spin()?;`
//!
//! Convenience: [`Node::new`] creates a private [`Context`] (fine for tcp/ipc).
//! Same-process **inproc** needs a shared [`crate::Context`] with the embedded
//! broker: `RobotBusBroker::start_with_context` + [`Node::inproc_with_context`].
//!
//! WebSocket RPC client mode (feature `ws`): [`Node::ws`] connects to the
//! broker gateway (subscribe / publish / call service / call action). No ZMQ
//! sockets; service and action **server** APIs return an error.
//!
//! Topic / service / action names are used as given (pass full paths yourself).

mod action_clients;
mod impls;
mod options;
mod publishers;
mod service_clients;

pub use action_clients::{
    GoalHandle, NodeActionClient, NodeActionClientRaw, NodeActionServer, RawActionFeedbackCallback,
    RawGoalHandle,
};
pub use options::NodeOptions;
pub use publishers::{TopicPublisher, TopicPublisherRaw};
pub use service_clients::{NodeService, NodeServiceClient, NodeServiceClientRaw};

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::errors::{BusError, Result};
use crate::message_bus::{Publisher as BusPublisher, Subscriber as BusSubscriber};
use crate::runtime::callback_group::{CallbackGroup, CallbackGroupType};
use crate::runtime::console_ready;
use crate::runtime::context::Context;
use crate::runtime::control_plane::ControlPlaneLedger;
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::executors::{ExecutorHandle, SingleThreadedExecutor};
use crate::runtime::parameters::{ListParametersResult, Parameter, ParameterStore, ParameterValue};
use crate::runtime::registrations::MessageCallback;
use crate::runtime::session::{BrokerSession, ConnectionState, SESSION_CREATE_WAIT};
use crate::runtime::topic_type_register;
use crate::runtime::topology_register::TopologyEndpointGuard;
use crate::zmq_helpers::HighWaterMark;
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::WsRuntime;

use options::ws_mode_unsupported;

pub struct Node {
    name: String,
    options: NodeOptions,
    context: Context,
    executor: Option<ExecutorHandle>,
    /// Keeps a lazily created [`SingleThreadedExecutor`] alive for the simple path.
    owned_executor: Option<SingleThreadedExecutor>,
    #[cfg(feature = "ws")]
    ws_runtime: Option<WsRuntime>,
    publisher: Option<Arc<BusPublisher>>,
    subscriber_connected: bool,
    default_callback_group: CallbackGroup,
    parameters: ParameterStore,
    /// Subscription id → topology guard (dropped on destroy_subscription).
    topology_subscriptions: HashMap<u64, Arc<TopologyEndpointGuard>>,
    /// Service server id → topology guard (dropped on destroy_service).
    topology_services: HashMap<u64, Arc<TopologyEndpointGuard>>,
    /// Action server id → topology guard (dropped on destroy_action_server).
    topology_actions: HashMap<u64, Arc<TopologyEndpointGuard>>,
    session: BrokerSession,
    control_plane: Arc<ControlPlaneLedger>,
}

#[cfg(feature = "ws")]
fn start_ws_runtime(options: &NodeOptions, session: &BrokerSession) -> Option<WsRuntime> {
    if !options.is_ws() {
        return None;
    }
    let url = options.resolved_ws_url().ok()?;
    match WsRuntime::new(url, Some(session.handle())) {
        Ok(rt) => Some(rt),
        Err(err) => {
            log::error!("failed to start ws runtime: {err}");
            None
        }
    }
}

impl Node {
    /// Convenience: tcp node with a **private** [`Context`].
    ///
    /// Prefer [`with_context`](Self::with_context) when aligning with ROS 2
    /// (`Context` → `Node`) or when multiple nodes should share sockets/inproc.
    ///
    /// Equivalent to [`Node::tcp`] (connects to `localhost` over TCP).
    pub fn new(name: impl Into<String>) -> Self {
        Self::tcp(name)
    }

    /// ROS 2–style preferred entry: share `context`, connect local broker over TCP.
    ///
    /// ```ignore
    /// let ctx = Context::new();
    /// let mut node = Node::with_context(&ctx, "pilot");
    /// ```
    ///
    /// For custom transports / endpoints, use [`with_context_options`](Self::with_context_options).
    pub fn with_context(context: &Context, name: impl Into<String>) -> Self {
        Self::with_context_options(context, name, NodeOptions::tcp())
    }

    /// TCP to the local broker (`localhost` + default ports).
    pub fn tcp(name: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::tcp())
    }

    /// TCP to a broker at `host` (default ports).
    pub fn tcp_at(name: impl Into<String>, host: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::tcp_at(host))
    }

    /// IPC under `/tmp/robot_bus` (default broker ipc binds).
    pub fn ipc(name: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::ipc())
    }

    /// IPC under a custom directory (must match the broker).
    pub fn ipc_at(name: impl Into<String>, dir: impl AsRef<str>) -> Self {
        Self::with_options(name, NodeOptions::ipc_at(dir))
    }

    /// Same-process `inproc://robot_bus/...`.
    ///
    /// For inproc to work, share a [`Context`] with the embedded broker via
    /// [`Node::inproc_with_context`] / [`Node::with_context_options`].
    pub fn inproc(name: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::inproc())
    }

    /// Same-process endpoints under a custom prefix (must match the broker).
    pub fn inproc_at(name: impl Into<String>, prefix: impl AsRef<str>) -> Self {
        Self::with_options(name, NodeOptions::inproc_at(prefix))
    }

    /// Same-process inproc using a shared [`Context`] (with the embedded broker).
    pub fn inproc_with_context(context: &Context, name: impl Into<String>) -> Self {
        Self::with_context_options(context, name, NodeOptions::inproc())
    }

    /// Same-process inproc under a custom prefix, sharing `context`.
    pub fn inproc_at_with_context(
        context: &Context,
        name: impl Into<String>,
        prefix: impl AsRef<str>,
    ) -> Self {
        Self::with_context_options(context, name, NodeOptions::inproc_at(prefix))
    }

    /// WebSocket RPC client node talking to the local broker gateway (`http://127.0.0.1:15570`).
    #[cfg(feature = "ws")]
    pub fn ws(name: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::ws())
    }

    /// WebSocket RPC client node talking to `url` (e.g. `http://127.0.0.1:15570`).
    #[cfg(feature = "ws")]
    pub fn ws_at(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::ws_at(url))
    }

    /// Create a node with explicit broker connection options (private ZMQ context).
    ///
    /// For `tcp` / `ipc` with unset endpoints, auto-discovers via
    /// `http://{host}:15570/api/v1/discover` (host `localhost` → `127.0.0.1`).
    pub fn with_options(name: impl Into<String>, options: NodeOptions) -> Self {
        let context = Context::new();
        Self::with_context_options(&context, name, options)
    }

    /// Create a node that shares `context` for all ZMQ sockets (required for inproc).
    ///
    /// Construction does not block on the broker. TCP/WS nodes discover in the
    /// background; use [`wait_for_broker`](Self::wait_for_broker) when startup
    /// must fail if the broker is down.
    pub fn with_context_options(
        context: &Context,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Self {
        let session = BrokerSession::start(options.clone());
        #[cfg(feature = "ws")]
        let ws_runtime = start_ws_runtime(&options, &session);
        let control_plane = ControlPlaneLedger::new();
        control_plane.update_endpoints(
            options.service_frontend.as_deref(),
            &options.host,
            &options.transport,
        );
        let node = Self {
            name: name.into(),
            options,
            context: context.clone(),
            executor: None,
            owned_executor: None,
            #[cfg(feature = "ws")]
            ws_runtime,
            publisher: None,
            subscriber_connected: false,
            default_callback_group: CallbackGroup::mutually_exclusive(),
            parameters: ParameterStore::new(),
            topology_subscriptions: HashMap::new(),
            topology_services: HashMap::new(),
            topology_actions: HashMap::new(),
            session,
            control_plane,
        };
        node.install_reconnect_hook();
        node
    }

    /// Shared runtime context used for ZMQ sockets.
    pub fn context(&self) -> &Context {
        &self.context
    }

    pub(crate) fn attach_executor(&mut self, handle: ExecutorHandle) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "WebSocket RPC node cannot attach to a ZMQ executor".into(),
            ));
        }
        if self.executor.is_some() {
            return Err(BusError::Protocol(
                "node is already added to an executor".into(),
            ));
        }
        self.executor = Some(handle);
        self.install_reconnect_hook();
        Ok(())
    }

    /// Return the attached executor, lazily creating a [`SingleThreadedExecutor`]
    /// when none was provided via [`add_node`](crate::runtime::SingleThreadedExecutor::add_node).
    fn ensure_executor(&mut self) -> Result<&ExecutorHandle> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "WebSocket RPC node does not use a ZMQ executor; use spin() on the node directly"
                    .into(),
            ));
        }
        if self.executor.is_none() {
            let exec = SingleThreadedExecutor::with_context(self.context.clone());
            self.attach_executor(exec.handle().clone())?;
            self.owned_executor = Some(exec);
        }
        Ok(self.executor.as_ref().expect("executor attached above"))
    }

    fn lock_executor(&mut self) -> Result<MutexGuard<'_, Executor>> {
        self.ensure_executor()?.lock()
    }

    #[cfg(feature = "ws")]
    fn ensure_ws(&mut self) -> Result<&WsRuntime> {
        if !self.options.is_ws() {
            return Err(BusError::Protocol(
                "internal: ensure_ws called on non-gRPC node".into(),
            ));
        }
        if self.ws_runtime.is_none() {
            let url = self.options.resolved_ws_url()?;
            self.ws_runtime = Some(WsRuntime::new(url, Some(self.session.handle()))?);
        }
        Ok(self.ws_runtime.as_ref().expect("ws runtime just created"))
    }

    fn pull_options(&mut self) {
        self.options = self.session.options();
        self.control_plane.update_endpoints(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
        );
    }

    fn install_reconnect_hook(&self) {
        let ledger = Arc::clone(&self.control_plane);
        let executor = self.executor.clone();
        let session = self.session.handle();
        self.session.set_reconnect_hook(Arc::new(move || {
            let options = session.options();
            ledger.update_endpoints(
                options.service_frontend.as_deref(),
                &options.host,
                &options.transport,
            );
            ledger.restore();
            if let Some(ref handle) = executor {
                if let Ok(mut exec) = handle.lock() {
                    exec.resend_worker_ready();
                }
            }
        }));
    }

    fn remember_topic_type(&self, topic: &str, type_name: &str) {
        self.control_plane.remember_topic_type(topic, type_name);
        topic_type_register::register_topic_type(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            topic,
            type_name,
        );
    }

    /// Wait until this node has a live broker control-plane (HTTP discover).
    ///
    /// Construction never throws on a missing broker. Scripts that must fail
    /// fast should call this after `Node::new`. `None` waits until Connected
    /// or [`shutdown`](Self::shutdown).
    pub fn wait_for_broker(&self, timeout: Option<Duration>) -> bool {
        self.session.wait_for_broker(timeout)
    }

    /// Current broker link state (see [`ConnectionState`]).
    pub fn connection_state(&self) -> ConnectionState {
        self.session.state()
    }

    /// Callback `(old, new, reason)` on every state change. Invoked from the
    /// session thread; keep it short and do not call back into the node.
    pub fn add_on_connection_event<F>(&self, callback: F)
    where
        F: Fn(ConnectionState, ConnectionState, &str) + Send + Sync + 'static,
    {
        self.session.add_on_connection_event(Arc::new(callback));
    }

    fn not_connected_err(&self) -> BusError {
        BusError::Protocol(format!(
            "node not connected to broker (state={}); call wait_for_broker() or start robot-bus-broker",
            self.session.state()
        ))
    }

    /// Pull discovered endpoints; for TCP/WS wait briefly so `create_*` still
    /// works when the broker is already up.
    fn ensure_connected(&mut self) -> Result<()> {
        self.pull_options();
        if self.session.state() == ConnectionState::Connected {
            return Ok(());
        }
        if !self.options.needs_endpoint_discover() && !self.options.is_ws() {
            return Ok(());
        }
        let _ = self.session.wait_for_broker(Some(SESSION_CREATE_WAIT));
        self.pull_options();
        if self.session.state() == ConnectionState::Connected {
            return Ok(());
        }
        if !self.options.needs_endpoint_discover() && !self.options.is_ws() {
            return Ok(());
        }
        Err(self.not_connected_err())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn options(&self) -> NodeOptions {
        self.session.options()
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

    /// Declare a local parameter with a default value (must not already exist).
    ///
    /// Returns the declared [`Parameter`] (ROS 2 `declare_parameter`).
    pub fn declare_parameter(
        &mut self,
        name: impl Into<String>,
        value: impl Into<ParameterValue>,
    ) -> Result<Parameter> {
        self.parameters.declare(name, value.into())
    }

    /// Read a previously declared parameter (ROS 2 `get_parameter` → [`Parameter`]).
    pub fn get_parameter(&self, name: &str) -> Result<Parameter> {
        self.parameters.get(name)
    }

    /// Read several parameters by name (ROS 2 `get_parameters`).
    pub fn get_parameters(&self, names: &[&str]) -> Result<Vec<Parameter>> {
        self.parameters.get_many(names)
    }

    /// Update a declared parameter (ROS 2 `set_parameter(rclcpp::Parameter(...))`).
    pub fn set_parameter(&mut self, parameter: Parameter) -> Result<()> {
        self.parameters.set_parameter(parameter)
    }

    /// Update several declared parameters (ROS 2 `set_parameters`).
    pub fn set_parameters(
        &mut self,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Result<()> {
        self.parameters.set_many(parameters)
    }

    /// Remove a declared parameter (ROS 2 `undeclare_parameter`).
    pub fn undeclare_parameter(&mut self, name: &str) -> Result<()> {
        self.parameters.undeclare(name)
    }

    /// Whether `name` has been declared.
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters.has(name)
    }

    /// List parameter names by prefix / depth (ROS 2 `list_parameters`).
    ///
    /// Empty `prefixes` lists the whole tree. `depth == 0` means recursive
    /// ([`crate::PARAMETER_DEPTH_RECURSIVE`]). Hierarchy separator is `.`.
    pub fn list_parameters(&self, prefixes: &[&str], depth: u64) -> ListParametersResult {
        self.parameters.list_parameters(prefixes, depth)
    }

    /// All declared parameters with values (convenience; not a ROS 2 API name).
    ///
    /// Prefer [`get_parameter`](Self::get_parameter) / [`get_parameters`](Self::get_parameters)
    /// when you already know the names.
    pub fn list_all_parameters(&self) -> Vec<Parameter> {
        self.parameters.list_all()
    }

    /// Load parameters from a YAML string (declare missing, set existing).
    ///
    /// Accepts a flat scalar map, or ROS 2–style `ros__parameters` /
    /// `"/**": { ros__parameters: … }`.
    pub fn load_parameters_from_yaml_str(&mut self, yaml: &str) -> Result<()> {
        self.parameters.load_from_yaml_str(yaml)
    }

    /// Load parameters from a YAML file.
    pub fn load_parameters_from_yaml_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        self.parameters.load_from_yaml_file(path)
    }

    pub fn shutdown_handle(&mut self) -> Result<ShutdownHandle> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return Ok(self.ensure_ws()?.shutdown_handle());
        }
        self.ensure_executor()?.shutdown_handle()
    }

    fn console_url_opt(&self) -> Option<String> {
        self.options
            .console_url
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                #[cfg(feature = "ws")]
                if self.options.is_ws() {
                    return self.options.ws_url.clone().filter(|s| !s.trim().is_empty());
                }
                None
            })
    }

    /// Console / API base used for readiness probes (`wait_for_service`, etc.).
    pub fn console_url(&self) -> String {
        console_ready::resolve_console_url(self.console_url_opt().as_deref())
    }

    /// Block until one message arrives on `topic`, or `timeout` elapses.
    ///
    /// ZMQ path uses a temporary SUB socket (dropped on return). gRPC/WS path
    /// registers a one-shot subscription callback and drives `spin_once`.
    pub fn wait_for_message(
        &mut self,
        topic: &str,
        timeout: Option<Duration>,
    ) -> Result<Option<Vec<u8>>> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.wait_for_message_ws(topic, timeout);
        }
        self.ensure_connected()?;
        let endpoint = self.options.message_xpub_endpoint()?;
        let sub = BusSubscriber::with_context_hwm(
            self.context.zmq(),
            Some(&endpoint),
            HighWaterMark::STREAM,
        )?;
        sub.subscribe(topic)?;
        match sub.receive(timeout) {
            Ok((_topic, payload)) => Ok(Some(payload)),
            Err(BusError::Timeout(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    #[cfg(feature = "ws")]
    fn wait_for_message_ws(
        &mut self,
        topic: &str,
        timeout: Option<Duration>,
    ) -> Result<Option<Vec<u8>>> {
        let slot: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
        let slot_cb = Arc::clone(&slot);
        let cb: MessageCallback = Arc::new(move |_topic, payload| {
            if let Ok(mut guard) = slot_cb.lock() {
                if guard.is_none() {
                    *guard = Some(payload.to_vec());
                }
            }
        });
        let handle = self.create_subscription_raw(topic, cb, None)?;
        let deadline = timeout.map(|d| Instant::now() + d);
        loop {
            if let Ok(guard) = slot.lock() {
                if guard.is_some() {
                    break;
                }
            }
            if let Some(deadline) = deadline {
                let now = Instant::now();
                if now >= deadline {
                    break;
                }
                let remaining = deadline.saturating_duration_since(now);
                let slice = remaining.min(Duration::from_millis(50));
                let _ = self.spin_once(Some(slice))?;
            } else {
                let _ = self.spin_once(Some(Duration::from_millis(50)))?;
            }
        }
        let _ = self.destroy_subscription(handle);
        let payload = slot
            .lock()
            .map_err(|_| BusError::Protocol("wait_for_message mutex poisoned".into()))?
            .take();
        Ok(payload)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.session.shutdown();
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            if let Some(ws) = self.ws_runtime.as_ref() {
                ws.shutdown();
            }
            return Ok(());
        }
        if self.executor.is_some() {
            return self.ensure_executor()?.shutdown();
        }
        Ok(())
    }

    /// Spin until [`shutdown`](Self::shutdown) (ROS 2–style `spin(node)`).
    ///
    /// Lazily creates a [`SingleThreadedExecutor`] when none was attached via
    /// `add_node`. Same as `executor.spin()` on the attached / owned executor.
    ///
    /// In WebSocket RPC mode, drives subscription callbacks and timers over the gateway.
    pub fn spin_once(&mut self, timeout: Option<Duration>) -> Result<bool> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.spin_once(timeout);
        }
        self.ensure_executor()?.spin_once(timeout)
    }

    pub fn spin_some(&mut self, timeout: Option<Duration>) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.spin_some(timeout);
        }
        self.ensure_executor()?.spin_some(timeout)
    }

    pub fn spin(&mut self) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.spin();
        }
        self.ensure_executor()?.spin()
    }

    pub fn start(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("start (use spin / spin_once)"));
        }
        self.ensure_executor()?.start()
    }

    pub fn stop(&mut self) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            if let Some(ws) = self.ws_runtime.as_ref() {
                ws.shutdown();
            }
            return Ok(());
        }
        self.ensure_executor()?.stop()
    }

    pub fn wait(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Ok(());
        }
        self.ensure_executor()?.wait()
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.session.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::runtime::SingleThreadedExecutor;

    #[test]
    fn node_local_parameters() {
        let mut node = Node::new("pilot");
        let declared = node
            .declare_parameter("max_speed", ParameterValue::Double(1.5))
            .unwrap();
        assert_eq!(declared.as_double().unwrap(), 1.5);
        node.declare_parameter("enabled", true).unwrap();
        assert_eq!(
            node.get_parameter("max_speed").unwrap().value,
            ParameterValue::Double(1.5)
        );
        assert_eq!(
            node.get_parameter("max_speed")
                .unwrap()
                .as_double()
                .unwrap(),
            1.5
        );
        node.set_parameter(Parameter::new("max_speed", 2.0))
            .unwrap();
        assert!(node.has_parameter("enabled"));
        assert_eq!(node.list_all_parameters().len(), 2);
        assert_eq!(
            node.list_parameters(&[], 0).names,
            vec!["enabled".to_string(), "max_speed".to_string()]
        );
        assert!(matches!(
            node.declare_parameter("enabled", false),
            Err(BusError::ParameterAlreadyDeclared { .. })
        ));
        assert!(matches!(
            node.set_parameter(Parameter::new("enabled", 1_i64)),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
        node.undeclare_parameter("enabled").unwrap();
        assert!(!node.has_parameter("enabled"));
    }

    #[test]
    fn node_load_parameters_from_yaml() {
        let mut node = Node::new("pilot");
        node.load_parameters_from_yaml_str(
            r#"
ros__parameters:
  max_speed: 1.25
  frame_id: base_link
"#,
        )
        .unwrap();
        assert_eq!(
            node.get_parameter("max_speed").unwrap().value,
            ParameterValue::Double(1.25)
        );
        node.load_parameters_from_yaml_str("max_speed: 3.0\n")
            .unwrap();
        assert_eq!(
            node.get_parameter("max_speed")
                .unwrap()
                .as_double()
                .unwrap(),
            3.0
        );
        let batch = node.get_parameters(&["max_speed", "frame_id"]).unwrap();
        assert_eq!(batch.len(), 2);
        node.set_parameters([
            Parameter::new("max_speed", 4.0),
            Parameter::new("frame_id", "odom"),
        ])
        .unwrap();
        assert_eq!(
            node.get_parameter("frame_id").unwrap().as_string().unwrap(),
            "odom"
        );
    }

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
    fn tcp_preset_leaves_endpoints_for_discover() {
        let opts = NodeOptions::tcp();
        assert_eq!(opts.transport, "tcp");
        assert!(opts.needs_endpoint_discover());
        assert!(opts.message_xsub_endpoint().is_err());
        let remote = NodeOptions::tcp_at("10.0.0.5");
        assert_eq!(remote.host, "10.0.0.5");
        assert!(remote.needs_endpoint_discover());
    }

    #[test]
    fn ipc_and_inproc_presets() {
        let ipc = NodeOptions::ipc();
        assert_eq!(ipc.transport, "ipc");
        assert_eq!(
            ipc.message_xsub_endpoint().unwrap(),
            "ipc:///tmp/robot_bus/message_bus_xsub.ipc"
        );

        let ipc_custom = NodeOptions::ipc_at("/var/run/robot_bus");
        assert_eq!(
            ipc_custom.service_frontend_endpoint().unwrap(),
            "ipc:///var/run/robot_bus/service_bus_frontend.ipc"
        );

        let inproc = NodeOptions::inproc();
        assert!(!inproc.needs_endpoint_discover());
        assert_eq!(
            inproc.message_xpub_endpoint().unwrap(),
            "inproc://robot_bus/message_bus/xpub"
        );

        let inproc_custom = NodeOptions::inproc_at("my_app");
        assert_eq!(
            inproc_custom.action_frontend_endpoint().unwrap(),
            "inproc://my_app/action_bus/frontend"
        );
    }

    #[test]
    fn node_transport_constructors() {
        assert_eq!(Node::tcp("a").options().transport, "tcp");
        assert_eq!(Node::tcp_at("a", "1.2.3.4").options().host, "1.2.3.4");
        assert_eq!(Node::ipc("a").options().transport, "ipc");
        assert_eq!(Node::inproc("a").options().transport, "inproc");
        assert_eq!(Node::new("a").options().transport, "tcp");
        let ctx = Context::new();
        assert_eq!(Node::with_context(&ctx, "a").options().transport, "tcp");
        assert_eq!(
            Node::with_context_options(&ctx, "a", NodeOptions::ipc())
                .options()
                .transport,
            "ipc"
        );
        #[cfg(feature = "ws")]
        {
            assert_eq!(Node::ws("a").options().transport, "ws");
            assert_eq!(
                Node::ws_at("a", "http://10.0.0.1:15570")
                    .options()
                    .ws_url
                    .as_deref(),
                Some("http://10.0.0.1:15570")
            );
            assert!(
                NodeOptions::ws()
                    .message_xpub_endpoint()
                    .unwrap_err()
                    .to_string()
                    .contains("WebSocket")
            );
        }
    }

    fn unit_node(name: &str) -> Node {
        Node::with_options(
            name,
            NodeOptions {
                message_xsub: Some("tcp://127.0.0.1:1".into()),
                message_xpub: Some("tcp://127.0.0.1:2".into()),
                service_frontend: Some("tcp://127.0.0.1:3".into()),
                service_backend: Some("tcp://127.0.0.1:4".into()),
                action_frontend: Some("tcp://127.0.0.1:5".into()),
                action_backend: Some("tcp://127.0.0.1:6".into()),
                ..NodeOptions::tcp()
            },
        )
    }

    #[test]
    fn add_node_then_create_publisher() {
        let mut node = unit_node("pilot");
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut node).unwrap();
        assert_eq!(node.name(), "pilot");
        let pub_ = node.create_publisher_raw("/robot1/imu").unwrap();
        assert_eq!(pub_.topic(), "/robot1/imu");
    }

    #[test]
    fn subscription_auto_attaches_single_threaded_executor() {
        let mut node = unit_node("pilot");
        node.create_subscription_raw("/imu", Arc::new(|_, _| {}), None)
            .unwrap();
        assert!(node.executor_handle().is_some());
        assert!(node.owned_executor.is_some());
    }

    #[test]
    fn spin_path_owns_single_threaded_executor() {
        let mut node = unit_node("pilot");
        // shutdown_handle / spin ensure the same lazy SingleThreadedExecutor.
        let _handle = node.shutdown_handle().unwrap();
        assert!(node.executor_handle().is_some());
        assert!(node.owned_executor.is_some());
    }

    #[test]
    fn cannot_add_node_after_auto_attach() {
        let mut node = unit_node("pilot");
        node.create_subscription_raw("/imu", Arc::new(|_, _| {}), None)
            .unwrap();
        let executor = SingleThreadedExecutor::new();
        let err = executor.add_node(&mut node).unwrap_err();
        assert!(err.to_string().contains("already added"));
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
