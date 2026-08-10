//! ROS 2–style [`Node`]: named participant with optional executor.
//!
//! Simple single-threaded flow (matches ROS 2 `spin(node)`):
//! 1. `let mut node = Node::new("pilot");`
//! 2. `let pub_ = node.create_publisher::<Imu>("/robot1/imu")?;`
//! 3. `node.spin()?;` — lazily owns a [`SingleThreadedExecutor`]
//!
//! For shared / multi-threaded executors: `executor.add_node(&mut node)?` then
//! `executor.spin()?`.
//!
//! Same-process **inproc** needs a shared [`crate::Context`] with the embedded
//! broker: `RobotBusBroker::start_with_context` + [`Node::inproc_with_context`].
//!
//! gRPC client mode (feature `grpc`): [`Node::ws`] connects to the broker
//! gateway only (subscribe / call service / call action). No ZMQ sockets;
//! publisher and server APIs return an error.
//!
//! Topic / service / action names are used as given (pass full paths yourself).

use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use prost::{Message, Name};

use crate::action_bus::{ActionClient as BusActionClient, ActionKind, ActionMessage};
use crate::errors::{BusError, Result, parse_error_body};
use crate::message_bus::Publisher as BusPublisher;
use crate::runtime::callback_group::{CallbackGroup, CallbackGroupType};
use crate::runtime::context::Context;
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::executors::{ExecutorHandle, SingleThreadedExecutor};
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::{WsClientContext, WsRuntime};
use crate::runtime::parameters::{Parameter, ParameterStore, ParameterValue};
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
use crate::runtime::timers::{TimerCallback, TimerHandle};
use crate::runtime::topic_type_register;
use crate::runtime::topology_register::TopologyEndpointGuard;
use crate::service_bus::ServiceClient as BusServiceClient;
use crate::transports::{
    ACTION_BACKEND_CHANNEL, ACTION_FRONTEND_CHANNEL, SERVICE_BACKEND_CHANNEL,
    SERVICE_FRONTEND_CHANNEL, XPUB_CHANNEL, XSUB_CHANNEL, inproc_endpoint_with_prefix,
    ipc_endpoint_in,
};
use crate::typed::{Action, ActionOutcome, Service};
use crate::zmq_helpers::HighWaterMark;

/// Broker connection settings owned by a [`Node`].
///
/// Defaults: `host = "localhost"`, `transport = "tcp"`. Prefer the presets
/// [`NodeOptions::tcp`] / [`NodeOptions::ipc`] / [`NodeOptions::inproc`]
/// (or [`Node::tcp`] / [`Node::ipc`] / [`Node::inproc`]) instead of filling
/// every endpoint by hand. Explicit endpoint fields still override derived
/// `transports::*` addresses when set.
///
/// For gateway-only clients, use [`NodeOptions::ws`] / [`Node::ws`]
/// (`transport = "ws"`, `ws_url` points at the broker gRPC listen address).
#[derive(Debug, Clone)]
pub struct NodeOptions {
    pub host: String,
    pub transport: String,
    /// gRPC gateway base URL when `transport == "ws"` (e.g. `http://127.0.0.1:15570`).
    pub ws_url: Option<String>,
    /// Embedded console HTTP base URL (same origin as gRPC when co-located).
    /// Filled by discovery when the broker announces it. Used by `rbus` / introspection
    /// clients; topology registration goes over the message bus.
    pub console_url: Option<String>,
    pub message_xsub: Option<String>,
    pub message_xpub: Option<String>,
    pub service_frontend: Option<String>,
    pub service_backend: Option<String>,
    pub action_backend: Option<String>,
    pub action_frontend: Option<String>,
}

impl Default for NodeOptions {
    fn default() -> Self {
        Self::tcp()
    }
}

impl NodeOptions {
    fn empty_endpoints(host: impl Into<String>, transport: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            transport: transport.into(),
            ws_url: None,
            console_url: None,
            message_xsub: None,
            message_xpub: None,
            service_frontend: None,
            service_backend: None,
            action_backend: None,
            action_frontend: None,
        }
    }

    /// TCP to the local broker (`localhost` + default ports).
    pub fn tcp() -> Self {
        Self::empty_endpoints("localhost", "tcp")
    }

    /// TCP to a broker at `host` (default ports).
    pub fn tcp_at(host: impl Into<String>) -> Self {
        Self::empty_endpoints(host, "tcp")
    }

    /// IPC under the default directory (`/tmp/robot_bus`). Prefer discover when the
    /// broker uses a broker-id namespaced ipc dir.
    pub fn ipc() -> Self {
        Self::ipc_at(crate::transports::IPC_DIR)
    }

    /// IPC under a custom directory (must match the broker's ipc binds).
    pub fn ipc_at(dir: impl AsRef<str>) -> Self {
        let dir = dir.as_ref();
        Self {
            host: "localhost".into(),
            transport: "ipc".into(),
            ws_url: None,
            console_url: None,
            message_xsub: Some(ipc_endpoint_in(dir, XSUB_CHANNEL)),
            message_xpub: Some(ipc_endpoint_in(dir, XPUB_CHANNEL)),
            service_frontend: Some(ipc_endpoint_in(dir, SERVICE_FRONTEND_CHANNEL)),
            service_backend: Some(ipc_endpoint_in(dir, SERVICE_BACKEND_CHANNEL)),
            action_backend: Some(ipc_endpoint_in(dir, ACTION_BACKEND_CHANNEL)),
            action_frontend: Some(ipc_endpoint_in(dir, ACTION_FRONTEND_CHANNEL)),
        }
    }

    /// Same-process `inproc://robot_bus/...` (default broker prefix).
    pub fn inproc() -> Self {
        Self::inproc_at("robot_bus")
    }

    /// Same-process endpoints under a custom prefix (must match the broker).
    ///
    /// `prefix` may be `my_app` or `inproc://my_app`.
    pub fn inproc_at(prefix: impl AsRef<str>) -> Self {
        let prefix = prefix.as_ref();
        Self {
            host: "localhost".into(),
            transport: "inproc".into(),
            ws_url: None,
            console_url: None,
            message_xsub: Some(inproc_endpoint_with_prefix(prefix, XSUB_CHANNEL)),
            message_xpub: Some(inproc_endpoint_with_prefix(prefix, XPUB_CHANNEL)),
            service_frontend: Some(inproc_endpoint_with_prefix(
                prefix,
                SERVICE_FRONTEND_CHANNEL,
            )),
            service_backend: Some(inproc_endpoint_with_prefix(prefix, SERVICE_BACKEND_CHANNEL)),
            action_backend: Some(inproc_endpoint_with_prefix(prefix, ACTION_BACKEND_CHANNEL)),
            action_frontend: Some(inproc_endpoint_with_prefix(prefix, ACTION_FRONTEND_CHANNEL)),
        }
    }

    /// gRPC gateway (native + browser WebSocket `/ws`) on the local broker (`http://127.0.0.1:15570`).
    #[cfg(feature = "ws")]
    pub fn ws() -> Self {
        Self::ws_at(WsRuntime::default_url())
    }

    /// gRPC gateway at `url` (e.g. `http://127.0.0.1:15570`); browsers use `ws(s)://…/ws`.
    #[cfg(feature = "ws")]
    pub fn ws_at(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            host: "127.0.0.1".into(),
            transport: "ws".into(),
            ws_url: Some(url),
            console_url: None,
            message_xsub: None,
            message_xpub: None,
            service_frontend: None,
            service_backend: None,
            action_backend: None,
            action_frontend: None,
        }
    }

    pub fn is_ws(&self) -> bool {
        self.transport == "ws"
    }

    fn require_zmq(&self) -> Result<()> {
        if self.is_ws() {
            Err(BusError::Protocol(
                "ZMQ endpoints are not available in gRPC node mode".into(),
            ))
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "ws")]
    pub fn resolved_ws_url(&self) -> Result<String> {
        if !self.is_ws() {
            return Err(BusError::Protocol(
                "ws_url is only valid when transport is \"ws\"".into(),
            ));
        }
        Ok(self
            .ws_url
            .clone()
            .unwrap_or_else(|| WsRuntime::default_url().to_string()))
    }

    pub fn message_xsub_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.message_xsub {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "message_xsub unset; call NodeOptions::discover(DiscoverOpts::default()) \
                 (GET http://127.0.0.1:15570/api/v1/discover) or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn message_xpub_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.message_xpub {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "message_xpub unset; call NodeOptions::discover(DiscoverOpts::default()) \
                 or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn service_frontend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.service_frontend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "service_frontend unset; call NodeOptions::discover(…) or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn service_backend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.service_backend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "service_backend unset; call NodeOptions::discover(…) or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn action_backend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.action_backend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "action_backend unset; call NodeOptions::discover(…) or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn action_frontend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.action_frontend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "action_frontend unset; call NodeOptions::discover(…) or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    /// True when ZMQ endpoint fields still need to be filled (e.g. via HTTP discover).
    pub fn needs_endpoint_discover(&self) -> bool {
        !self.is_ws()
            && self.message_xsub.is_none()
            && self.message_xpub.is_none()
            && self.service_frontend.is_none()
            && self.service_backend.is_none()
            && self.action_frontend.is_none()
            && self.action_backend.is_none()
            && self.transport != "inproc"
    }
}

fn grpc_mode_unsupported(op: &str) -> BusError {
    BusError::Protocol(format!(
        "{op} is not supported in gRPC node mode (client: subscribe / publish / call service / call action; no servers)"
    ))
}

/// Raw (opaque bytes) publisher from [`Node::create_publisher_raw`].
///
/// ZMQ mode shares one underlying bus PUB socket per node; gRPC mode issues
/// unary `MessageGateway.Publish` RPCs. Each handle remembers its topic.
///
/// Topology registration is shared across clones and unregistered when the last
/// handle drops.
#[derive(Clone)]
pub struct TopicPublisherRaw {
    backend: TopicPublisherBackend,
    topic: String,
    /// Best-effort console topology registration (kept alive while handles exist).
    _topology: Option<Arc<TopologyEndpointGuard>>,
}

#[derive(Clone)]
enum TopicPublisherBackend {
    Zmq(Arc<BusPublisher>),
    #[cfg(feature = "ws")]
    Ws(WsClientContext),
}

impl TopicPublisherRaw {
    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn publish(&self, payload: &[u8]) -> Result<()> {
        match &self.backend {
            TopicPublisherBackend::Zmq(inner) => inner.publish(&self.topic, payload),
            #[cfg(feature = "ws")]
            TopicPublisherBackend::Ws(ctx) => ctx.publish(&self.topic, payload),
        }
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        match &self.backend {
            TopicPublisherBackend::Zmq(inner) => inner.high_water_mark(),
            #[cfg(feature = "ws")]
            TopicPublisherBackend::Ws(_) => Ok(HighWaterMark::STREAM),
        }
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        match &self.backend {
            TopicPublisherBackend::Zmq(inner) => inner.set_high_water_mark(hwm),
            #[cfg(feature = "ws")]
            TopicPublisherBackend::Ws(_) => Err(BusError::Protocol(
                "set_high_water_mark is not available for gRPC publishers".into(),
            )),
        }
    }
}

/// Typed topic publisher returned by [`Node::create_publisher`] (ROS 2 style).
#[derive(Clone)]
pub struct TopicPublisher<M: Message + Default> {
    inner: TopicPublisherRaw,
    _marker: PhantomData<M>,
}

impl<M: Message + Default> TopicPublisher<M> {
    pub fn topic(&self) -> &str {
        self.inner.topic()
    }

    pub fn publish(&self, msg: &M) -> Result<()> {
        self.inner.publish(&msg.encode_to_vec())
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.inner.high_water_mark()
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.inner.set_high_water_mark(hwm)
    }
}

/// Service server handle returned by [`Node::create_service`] / [`Node::create_service_raw`].
#[derive(Clone, Debug)]
pub struct NodeService {
    service_name: String,
}

impl NodeService {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

/// Raw (opaque bytes) service client from [`Node::create_client_raw`].
pub struct NodeServiceClientRaw {
    inner: ServiceClientInner,
    service_name: String,
}

enum ServiceClientInner {
    Zmq(BusServiceClient),
    #[cfg(feature = "ws")]
    Ws(WsClientContext),
}

impl NodeServiceClientRaw {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn call(&self, body: &[u8], timeout: Option<Duration>) -> Result<Vec<u8>> {
        self.call_with_id(body, None, timeout)
    }

    pub fn call_with_id(
        &self,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        match &self.inner {
            ServiceClientInner::Zmq(client) => {
                client.call(&self.service_name, body, request_id, timeout)
            }
            #[cfg(feature = "ws")]
            ServiceClientInner::Ws(ctx) => {
                ctx.call_service(&self.service_name, body, request_id, timeout)
            }
        }
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        match &self.inner {
            ServiceClientInner::Zmq(client) => client.high_water_mark(),
            #[cfg(feature = "ws")]
            ServiceClientInner::Ws(_) => Err(BusError::Protocol(
                "high_water_mark is not available in gRPC node mode".into(),
            )),
        }
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        match &self.inner {
            ServiceClientInner::Zmq(client) => client.set_high_water_mark(hwm),
            #[cfg(feature = "ws")]
            ServiceClientInner::Ws(_) => Err(BusError::Protocol(
                "set_high_water_mark is not available in gRPC node mode".into(),
            )),
        }
    }
}

/// Typed service client returned by [`Node::create_client`] (ROS 2 / rclrs style).
pub struct NodeServiceClient<S: Service> {
    inner: NodeServiceClientRaw,
    _marker: PhantomData<S>,
}

impl<S: Service> NodeServiceClient<S> {
    pub fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    pub fn call(&self, request: &S::Request, timeout: Option<Duration>) -> Result<S::Response> {
        let reply = self.inner.call(&request.encode_to_vec(), timeout)?;
        S::Response::decode(reply.as_slice()).map_err(|err| {
            BusError::Protocol(format!(
                "service '{}' response decode failed: {err}",
                self.service_name()
            ))
        })
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.inner.high_water_mark()
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.inner.set_high_water_mark(hwm)
    }
}

/// Action server handle returned by [`Node::create_action_server`] /
/// [`Node::create_action_server_raw`].
#[derive(Clone, Debug)]
pub struct NodeActionServer {
    action_name: String,
}

impl NodeActionServer {
    pub fn action_name(&self) -> &str {
        &self.action_name
    }
}

/// Callback invoked for each action feedback as it arrives.
pub type RawActionFeedbackCallback = Arc<dyn Fn(&ActionMessage) + Send + Sync + 'static>;

fn spawn_zmq_goal(
    context: zmq::Context,
    endpoint: String,
    action_name: String,
    body: Vec<u8>,
    requested_goal_id: Option<String>,
    timeout: Option<Duration>,
    hwm: HighWaterMark,
    feedback_callback: Option<RawActionFeedbackCallback>,
) -> Result<RawGoalHandle> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (event_tx, event_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    let thread_action_name = action_name.clone();

    thread::Builder::new()
        .name(format!("action-{}", action_name))
        .spawn(move || {
            let client = match BusActionClient::with_context_hwm(&context, Some(&endpoint), hwm) {
                Ok(client) => client,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
            let goal_id = match client.submit_goal(
                &thread_action_name,
                &body,
                requested_goal_id.as_deref(),
            ) {
                Ok(goal_id) => goal_id,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
            if ready_tx.send(Ok(goal_id.clone())).is_err() {
                return;
            }

            let deadline = timeout.map(|duration| Instant::now() + duration);
            loop {
                while let Ok(command) = command_rx.try_recv() {
                    match command {
                        GoalCommand::Cancel(body) => {
                            if let Err(err) =
                                client.submit_cancel(&thread_action_name, &goal_id, &body)
                            {
                                let _ = event_tx.send(Err(err));
                                return;
                            }
                        }
                    }
                }

                if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    let _ = client.submit_cancel(&thread_action_name, &goal_id, b"");
                    let _ = event_tx.send(Err(BusError::Timeout(format!(
                        "action client timed out after {}s",
                        timeout.unwrap_or_default().as_secs_f64()
                    ))));
                    return;
                }

                let poll_timeout = deadline
                    .map(|deadline| {
                        deadline
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(20))
                    })
                    .unwrap_or(Duration::from_millis(20));
                let message = match client.recv_message(Some(poll_timeout)) {
                    Ok(message) => message,
                    Err(BusError::Timeout(_)) => continue,
                    Err(err) => {
                        let _ = event_tx.send(Err(err));
                        return;
                    }
                };
                if message.action_name != thread_action_name || message.goal_id != goal_id {
                    let _ = event_tx.send(Err(BusError::Protocol(format!(
                        "unexpected message for {:?}/{:?}",
                        message.action_name, message.goal_id
                    ))));
                    return;
                }
                if message.kind == ActionKind::Feedback {
                    if let Some(callback) = &feedback_callback {
                        callback(&message);
                    }
                }
                let done = message.kind == ActionKind::Result;
                if done {
                    if let Some(err) = parse_error_body(&message.body) {
                        let _ = event_tx.send(Err(err));
                        return;
                    }
                }
                if event_tx.send(Ok(message)).is_err() || done {
                    return;
                }
            }
        })
        .map_err(|err| BusError::Protocol(format!("spawn action thread: {err}")))?;

    let goal_id = ready_rx
        .recv()
        .map_err(|_| BusError::Protocol("action thread ended before submitting goal".into()))??;
    Ok(RawGoalHandle {
        inner: Arc::new(GoalHandleCore {
            action_name,
            goal_id,
            events: Mutex::new(event_rx),
            messages: Mutex::new(Vec::new()),
            control: GoalControl::Zmq(command_tx),
            completed: AtomicBool::new(false),
        }),
    })
}

enum GoalControl {
    Zmq(Sender<GoalCommand>),
    #[cfg(feature = "ws")]
    Ws(crate::runtime::ws_runtime::WsCancelHandle),
}

enum GoalCommand {
    Cancel(Vec<u8>),
}

struct GoalHandleCore {
    action_name: String,
    goal_id: String,
    events: Mutex<Receiver<Result<ActionMessage>>>,
    messages: Mutex<Vec<ActionMessage>>,
    control: GoalControl,
    completed: AtomicBool,
}

impl GoalHandleCore {
    fn wait_result(&self) -> Result<ActionMessage> {
        if let Some(result) = self
            .messages
            .lock()
            .map_err(|_| BusError::Protocol("action messages mutex poisoned".into()))?
            .iter()
            .find(|message| message.kind == ActionKind::Result)
            .cloned()
        {
            return Ok(result);
        }

        loop {
            let event = self
                .events
                .lock()
                .map_err(|_| BusError::Protocol("action event mutex poisoned".into()))?
                .recv()
                .map_err(|_| {
                    BusError::Protocol(format!(
                        "action '{}' goal '{}' ended without RESULT",
                        self.action_name, self.goal_id
                    ))
                })??;
            let done = event.kind == ActionKind::Result;
            self.messages
                .lock()
                .map_err(|_| BusError::Protocol("action messages mutex poisoned".into()))?
                .push(event.clone());
            if done {
                self.completed.store(true, Ordering::Release);
                return Ok(event);
            }
        }
    }

    fn collect(&self) -> Result<Vec<ActionMessage>> {
        self.wait_result()?;
        self.messages
            .lock()
            .map(|messages| messages.clone())
            .map_err(|_| BusError::Protocol("action messages mutex poisoned".into()))
    }

    fn cancel(&self, body: &[u8]) -> Result<()> {
        match &self.control {
            GoalControl::Zmq(commands) => commands
                .send(GoalCommand::Cancel(body.to_vec()))
                .map_err(|_| BusError::Closed),
            #[cfg(feature = "ws")]
            GoalControl::Ws(abort) => {
                abort.abort();
                Ok(())
            }
        }
    }
}

impl Drop for GoalHandleCore {
    fn drop(&mut self) {
        if self.completed.load(Ordering::Acquire) {
            return;
        }
        match &self.control {
            GoalControl::Zmq(commands) => {
                let _ = commands.send(GoalCommand::Cancel(Vec::new()));
            }
            #[cfg(feature = "ws")]
            GoalControl::Ws(abort) => abort.abort(),
        }
    }
}

/// Live handle for one raw (opaque bytes) action goal.
#[derive(Clone)]
pub struct RawGoalHandle {
    inner: Arc<GoalHandleCore>,
}

impl RawGoalHandle {
    pub fn goal_id(&self) -> &str {
        &self.inner.goal_id
    }

    pub fn action_name(&self) -> &str {
        &self.inner.action_name
    }

    pub fn wait_result(&self) -> Result<ActionMessage> {
        self.inner.wait_result()
    }

    pub fn collect(&self) -> Result<Vec<ActionMessage>> {
        self.inner.collect()
    }

    /// Best-effort cancellation. This does not wait for server acknowledgement.
    pub fn cancel(&self) -> Result<()> {
        self.inner.cancel(&[])
    }

    /// Best-effort cancellation with an opaque ZMQ cancel payload.
    ///
    /// On native gRPC this aborts the response stream (body ignored). Browser
    /// WebSocket clients send an explicit CANCEL frame instead.
    pub fn cancel_with_body(&self, body: &[u8]) -> Result<()> {
        self.inner.cancel(body)
    }
}

/// Live handle for one typed action goal.
pub struct GoalHandle<A: Action> {
    inner: RawGoalHandle,
    _marker: PhantomData<A>,
}

impl<A: Action> Clone for GoalHandle<A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<A: Action> GoalHandle<A> {
    pub fn goal_id(&self) -> &str {
        self.inner.goal_id()
    }

    pub fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    pub fn wait_result(&self) -> Result<A::Result> {
        let message = self.inner.wait_result()?;
        A::Result::decode(message.body.as_slice()).map_err(|err| {
            BusError::Protocol(format!(
                "action '{}' result decode failed: {err}",
                self.action_name()
            ))
        })
    }

    pub fn cancel(&self) -> Result<()> {
        self.inner.cancel()
    }
}

/// Raw (opaque bytes) action client from [`Node::create_action_client_raw`].
pub struct NodeActionClientRaw {
    inner: ActionClientInner,
    action_name: String,
}

enum ActionClientInner {
    Zmq {
        context: zmq::Context,
        endpoint: String,
        hwm: Mutex<HighWaterMark>,
    },
    #[cfg(feature = "ws")]
    Ws(WsClientContext),
}

impl NodeActionClientRaw {
    pub fn action_name(&self) -> &str {
        &self.action_name
    }

    pub fn send_goal(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
        feedback_callback: Option<RawActionFeedbackCallback>,
    ) -> Result<RawGoalHandle> {
        match &self.inner {
            ActionClientInner::Zmq {
                context,
                endpoint,
                hwm,
            } => {
                let hwm = *hwm
                    .lock()
                    .map_err(|_| BusError::Protocol("action HWM mutex poisoned".into()))?;
                spawn_zmq_goal(
                    context.clone(),
                    endpoint.clone(),
                    self.action_name.clone(),
                    body.to_vec(),
                    goal_id.map(str::to_string),
                    timeout,
                    hwm,
                    feedback_callback,
                )
            }
            #[cfg(feature = "ws")]
            ActionClientInner::Ws(ctx) => ctx
                .send_goal(&self.action_name, body, goal_id, timeout, feedback_callback)
                .map(|session| RawGoalHandle {
                    inner: Arc::new(GoalHandleCore {
                        action_name: self.action_name.clone(),
                        goal_id: session.goal_id,
                        events: Mutex::new(session.events),
                        messages: Mutex::new(Vec::new()),
                        control: GoalControl::Ws(session.abort),
                        completed: AtomicBool::new(false),
                    }),
                }),
        }
    }

    /// Compatibility helper that waits for and collects FEEDBACK/RESULT.
    pub fn send_goal_and_wait(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        self.send_goal(body, goal_id, timeout, None)?.collect()
    }

    /// Alias for [`send_goal_and_wait`](Self::send_goal_and_wait).
    pub fn collect(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        self.send_goal_and_wait(body, goal_id, timeout)
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        match &self.inner {
            ActionClientInner::Zmq { hwm, .. } => hwm
                .lock()
                .map(|hwm| *hwm)
                .map_err(|_| BusError::Protocol("action HWM mutex poisoned".into())),
            #[cfg(feature = "ws")]
            ActionClientInner::Ws(_) => Err(BusError::Protocol(
                "high_water_mark is not available in gRPC node mode".into(),
            )),
        }
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        match &self.inner {
            ActionClientInner::Zmq { hwm: current, .. } => {
                *current
                    .lock()
                    .map_err(|_| BusError::Protocol("action HWM mutex poisoned".into()))? = hwm;
                Ok(())
            }
            #[cfg(feature = "ws")]
            ActionClientInner::Ws(_) => Err(BusError::Protocol(
                "set_high_water_mark is not available in gRPC node mode".into(),
            )),
        }
    }
}

/// Typed action client returned by [`Node::create_action_client`] (ROS 2 style).
pub struct NodeActionClient<A: Action> {
    inner: NodeActionClientRaw,
    _marker: PhantomData<A>,
}

impl<A: Action> NodeActionClient<A> {
    pub fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    pub fn send_goal(
        &self,
        goal: &A::Goal,
        goal_id: Option<&str>,
        timeout: Option<Duration>,
        feedback_callback: Option<Arc<dyn Fn(A::Feedback) + Send + Sync + 'static>>,
    ) -> Result<GoalHandle<A>> {
        let action_name = self.action_name().to_string();
        let raw_callback = feedback_callback.map(|callback| {
            Arc::new(move |message: &ActionMessage| {
                match A::Feedback::decode(message.body.as_slice()) {
                    Ok(feedback) => callback(feedback),
                    Err(err) => {
                        log::warn!("action '{}' feedback decode failed: {err}", action_name)
                    }
                }
            }) as RawActionFeedbackCallback
        });
        let inner = self
            .inner
            .send_goal(&goal.encode_to_vec(), goal_id, timeout, raw_callback)?;
        Ok(GoalHandle {
            inner,
            _marker: PhantomData,
        })
    }

    /// Compatibility helper that waits for a result and collects feedback.
    pub fn send_goal_and_wait(
        &self,
        goal: &A::Goal,
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<ActionOutcome<A>> {
        let messages = self
            .inner
            .send_goal_and_wait(&goal.encode_to_vec(), goal_id, timeout)?;
        let mut feedbacks = Vec::new();
        let mut result = None;
        for msg in messages {
            match msg.kind {
                ActionKind::Feedback => {
                    let fb = A::Feedback::decode(msg.body.as_slice()).map_err(|err| {
                        BusError::Protocol(format!(
                            "action '{}' feedback decode failed: {err}",
                            self.action_name()
                        ))
                    })?;
                    feedbacks.push(fb);
                }
                ActionKind::Result => {
                    let res = A::Result::decode(msg.body.as_slice()).map_err(|err| {
                        BusError::Protocol(format!(
                            "action '{}' result decode failed: {err}",
                            self.action_name()
                        ))
                    })?;
                    result = Some(res);
                }
                ActionKind::Goal | ActionKind::Cancel => {}
            }
        }
        let result = result.ok_or_else(|| {
            BusError::Protocol(format!(
                "action '{}' completed without RESULT",
                self.action_name()
            ))
        })?;
        Ok(ActionOutcome { feedbacks, result })
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
/// Create with [`Node::new`], then `create_*` and [`spin`](Self::spin). A
/// [`SingleThreadedExecutor`] is created automatically on first use. For a
/// shared or multi-threaded executor, call
/// [`SingleThreadedExecutor::add_node`](crate::runtime::SingleThreadedExecutor::add_node)
/// (or the multi-threaded variant) before `create_*` / `spin`.
///
/// gRPC mode ([`Node::ws`]): client-only over the broker gateway; does not
/// attach to a ZMQ executor.
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
    /// Subscription topology guards (no per-sub handle; live until node drop).
    topology_subscriptions: Vec<Arc<TopologyEndpointGuard>>,
}

impl Node {
    /// Create a node that is not yet attached to an executor.
    ///
    /// Equivalent to [`Node::tcp`] (connects to `localhost` over TCP).
    pub fn new(name: impl Into<String>) -> Self {
        Self::tcp(name)
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
    /// [`Node::inproc_with_context`] / [`Node::with_context`].
    pub fn inproc(name: impl Into<String>) -> Self {
        Self::with_options(name, NodeOptions::inproc())
    }

    /// Same-process endpoints under a custom prefix (must match the broker).
    pub fn inproc_at(name: impl Into<String>, prefix: impl AsRef<str>) -> Self {
        Self::with_options(name, NodeOptions::inproc_at(prefix))
    }

    /// Same-process inproc using a shared [`Context`] (with the embedded broker).
    pub fn inproc_with_context(context: Context, name: impl Into<String>) -> Self {
        Self::with_context(context, name, NodeOptions::inproc())
    }

    /// Same-process inproc under a custom prefix, sharing `context`.
    pub fn inproc_at_with_context(
        context: Context,
        name: impl Into<String>,
        prefix: impl AsRef<str>,
    ) -> Self {
        Self::with_context(context, name, NodeOptions::inproc_at(prefix))
    }

    /// gRPC client node talking to the local broker gateway (`http://127.0.0.1:15570`).
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
        Self::with_context(Context::new(), name, options)
    }

    /// Create a node that shares `context` for all ZMQ sockets (required for inproc).
    pub fn with_context(context: Context, name: impl Into<String>, mut options: NodeOptions) -> Self {
        if options.needs_endpoint_discover() {
            let discover_opts = crate::discovery::DiscoverOpts::for_host(&options.host);
            match options.clone().discover(discover_opts) {
                Ok(filled) => options = filled,
                Err(err) => {
                    log::warn!(
                        "auto-discover failed for node (start broker / check --api-listen): {err}"
                    );
                }
            }
        }
        Self {
            name: name.into(),
            options,
            context,
            executor: None,
            owned_executor: None,
            #[cfg(feature = "ws")]
            ws_runtime: None,
            publisher: None,
            subscriber_connected: false,
            default_callback_group: CallbackGroup::mutually_exclusive(),
            parameters: ParameterStore::new(),
            topology_subscriptions: Vec::new(),
        }
    }

    /// Shared runtime context used for ZMQ sockets.
    pub fn context(&self) -> &Context {
        &self.context
    }

    pub(crate) fn attach_executor(&mut self, handle: ExecutorHandle) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "gRPC node cannot attach to a ZMQ executor".into(),
            ));
        }
        if self.executor.is_some() {
            return Err(BusError::Protocol(
                "node is already added to an executor".into(),
            ));
        }
        self.executor = Some(handle);
        Ok(())
    }

    /// Return the attached executor, lazily creating a [`SingleThreadedExecutor`]
    /// when none was provided via [`add_node`](crate::runtime::SingleThreadedExecutor::add_node).
    fn ensure_executor(&mut self) -> Result<&ExecutorHandle> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "gRPC node does not use a ZMQ executor; use spin() on the node directly".into(),
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
            self.ws_runtime = Some(WsRuntime::new(url)?);
        }
        Ok(self.ws_runtime.as_ref().expect("ws runtime just created"))
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

    /// Declare a local parameter with a default value (must not already exist).
    pub fn declare_parameter(
        &mut self,
        name: impl Into<String>,
        value: ParameterValue,
    ) -> Result<()> {
        self.parameters.declare(name, value)
    }

    /// Read a previously declared parameter.
    pub fn get_parameter(&self, name: &str) -> Result<ParameterValue> {
        self.parameters.get(name)
    }

    /// Update a declared parameter (type must match the declared variant).
    pub fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
        self.parameters.set(name, value)
    }

    /// Whether `name` has been declared.
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters.has(name)
    }

    /// All declared parameters, sorted by name.
    pub fn list_parameters(&self) -> Vec<Parameter> {
        self.parameters.list()
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

    /// Create a typed topic publisher (ROS 2 `create_publisher`).
    ///
    /// Multiple publishers on the same node share one bus PUB socket.
    /// Best-effort registers `topic → M::full_name()` with the broker console.
    pub fn create_publisher<M: Message + Name + Default>(
        &mut self,
        topic: impl Into<String>,
    ) -> Result<TopicPublisher<M>> {
        let topic = topic.into();
        let pub_ = TopicPublisher {
            inner: self.create_publisher_raw(topic.clone())?,
            _marker: PhantomData,
        };
        topic_type_register::register_topic_type(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &topic,
            &M::full_name(),
        );
        Ok(pub_)
    }

    /// Create a raw-bytes topic publisher.
    pub fn create_publisher_raw(&mut self, topic: impl Into<String>) -> Result<TopicPublisherRaw> {
        self.create_publisher_raw_with_hwm(topic, None)
    }

    /// Like [`create_publisher_raw`](Self::create_publisher_raw), optionally setting HWM
    /// on first socket connect.
    pub fn create_publisher_raw_with_hwm(
        &mut self,
        topic: impl Into<String>,
        hwm: Option<HighWaterMark>,
    ) -> Result<TopicPublisherRaw> {
        let topic = topic.into();
        crate::console_topics::check_not_reserved(&topic)?;
        let topology = Some(TopologyEndpointGuard::start(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &self.name,
            "publisher",
            &topic,
        ));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let _ = hwm; // gRPC publish has no local ZMQ HWM
            let grpc = self.ensure_ws()?;
            return Ok(TopicPublisherRaw {
                backend: TopicPublisherBackend::Ws(grpc.client_context()),
                topic,
                _topology: topology,
            });
        }
        self.ensure_bus_publisher(hwm)?;
        Ok(TopicPublisherRaw {
            backend: TopicPublisherBackend::Zmq(Arc::clone(
                self.publisher.as_ref().expect("publisher just ensured"),
            )),
            topic,
            _topology: topology,
        })
    }

    /// Like [`create_publisher`](Self::create_publisher), setting HWM on first socket connect.
    pub fn create_publisher_with_hwm<M: Message + Name + Default>(
        &mut self,
        topic: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<TopicPublisher<M>> {
        let topic = topic.into();
        let pub_ = TopicPublisher {
            inner: self.create_publisher_raw_with_hwm(topic.clone(), Some(hwm))?,
            _marker: PhantomData,
        };
        topic_type_register::register_topic_type(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &topic,
            &M::full_name(),
        );
        Ok(pub_)
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
        self.publisher = Some(Arc::new(BusPublisher::with_context_hwm(
            self.context.zmq(),
            Some(&endpoint),
            hwm,
        )?));
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

    pub fn stream_hwm(&mut self) -> Result<HighWaterMark> {
        if self.options.is_ws() {
            return Ok(HighWaterMark::STREAM);
        }
        Ok(self.lock_executor()?.stream_hwm())
    }

    pub fn set_stream_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "set_stream_hwm is not available in gRPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_stream_hwm(hwm)
    }

    pub fn rpc_hwm(&mut self) -> Result<HighWaterMark> {
        if self.options.is_ws() {
            return Ok(HighWaterMark::RPC);
        }
        Ok(self.lock_executor()?.rpc_hwm())
    }

    pub fn set_rpc_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "set_rpc_hwm is not available in gRPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_rpc_hwm(hwm)
    }

    pub fn action_hwm(&mut self) -> Result<HighWaterMark> {
        if self.options.is_ws() {
            return Ok(HighWaterMark::ACTION);
        }
        Ok(self.lock_executor()?.action_hwm())
    }

    pub fn set_action_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "set_action_hwm is not available in gRPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_action_hwm(hwm)
    }

    /// Subscribe with a protobuf-typed callback (ROS 2 `create_subscription`).
    ///
    /// Decode failures are skipped (logged). `callback_group: None` uses the
    /// node's default mutually exclusive group.
    /// Best-effort registers `topic → M::full_name()` with the broker console.
    pub fn create_subscription<M, F>(
        &mut self,
        topic: &str,
        callback: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<()>
    where
        M: Message + Name + Default + 'static,
        F: Fn(&str, M) + Send + Sync + 'static,
    {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let cb: MessageCallback = Arc::new(move |topic, payload| match M::decode(payload) {
            Ok(msg) => callback(topic, msg),
            Err(err) => log::warn!("typed subscription decode failed: {err}"),
        });
        self.create_subscription_raw(topic, cb, Some(&group))?;
        topic_type_register::register_topic_type(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            topic,
            &M::full_name(),
        );
        Ok(())
    }

    /// Subscribe with a raw-bytes callback.
    pub fn create_subscription_raw(
        &mut self,
        topic: &str,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<()> {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let topology = TopologyEndpointGuard::start(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &self.name,
            "subscriber",
            topic,
        );
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            self.ensure_ws()?.subscribe(topic, callback, group)?;
            self.topology_subscriptions.push(topology);
            return Ok(());
        }
        self.ensure_subscriber()?;
        self.lock_executor()?.subscribe(topic, callback, group)?;
        self.topology_subscriptions.push(topology);
        Ok(())
    }

    fn ensure_subscriber(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "internal: ensure_subscriber on gRPC node".into(),
            ));
        }
        if !self.subscriber_connected {
            let endpoint = self.options.message_xpub_endpoint()?;
            self.lock_executor()?.connect_subscriber(Some(&endpoint))?;
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
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.create_timer(period, callback, group);
        }
        self.lock_executor()?.create_timer(period, callback, group)
    }

    pub fn cancel_timer(&mut self, handle: TimerHandle) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.cancel_timer(handle);
        }
        self.lock_executor()?.cancel_timer(handle)
    }

    /// Register a typed service server (ROS 2 / rclrs `create_service`).
    ///
    /// Decode failures log a warning and return an empty response body.
    pub fn create_service<S, F>(
        &mut self,
        service_name: &str,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService>
    where
        S: Service,
        F: Fn(S::Request) -> S::Response + Send + Sync + 'static,
    {
        let cb: ServiceHandler = Arc::new(move |body| match S::Request::decode(body) {
            Ok(req) => handler(req).encode_to_vec(),
            Err(err) => {
                log::warn!("typed service decode failed: {err}");
                Vec::new()
            }
        });
        self.create_service_raw(service_name, cb, callback_group)
    }

    /// Register a raw-bytes service server.
    pub fn create_service_raw(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService> {
        crate::console_topics::check_not_reserved(service_name)?;
        if self.options.is_ws() {
            return Err(grpc_mode_unsupported("create_service"));
        }
        let endpoint = self.options.service_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?.register_service(
            service_name,
            handler,
            group,
            Some(&endpoint),
            None,
        )?;
        Ok(NodeService {
            service_name: service_name.to_string(),
        })
    }

    /// Create a typed service client (ROS 2 / rclrs `create_client`).
    pub fn create_client<S: Service>(
        &mut self,
        service_name: impl Into<String>,
    ) -> Result<NodeServiceClient<S>> {
        Ok(NodeServiceClient {
            inner: self.create_client_raw(service_name)?,
            _marker: PhantomData,
        })
    }

    /// Create a raw-bytes service client bound to `service_name`.
    pub fn create_client_raw(
        &mut self,
        service_name: impl Into<String>,
    ) -> Result<NodeServiceClientRaw> {
        self.create_client_raw_with_hwm(service_name, self.client_rpc_hwm())
    }

    /// Like [`create_client_raw`](Self::create_client_raw), with an explicit HWM.
    pub fn create_client_raw_with_hwm(
        &mut self,
        service_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeServiceClientRaw> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeServiceClientRaw {
                inner: ServiceClientInner::Ws(ctx),
                service_name: service_name.into(),
            });
        }
        let endpoint = self.options.service_frontend_endpoint()?;
        Ok(NodeServiceClientRaw {
            inner: ServiceClientInner::Zmq(BusServiceClient::with_context_hwm(
                self.context.zmq(),
                Some(&endpoint),
                hwm,
            )?),
            service_name: service_name.into(),
        })
    }

    fn client_rpc_hwm(&self) -> HighWaterMark {
        match &self.executor {
            Some(exec) => exec
                .lock()
                .map(|e| e.rpc_hwm())
                .unwrap_or(HighWaterMark::RPC),
            None => HighWaterMark::RPC,
        }
    }

    /// Register a typed action server (ROS 2–style `create_action_server`).
    pub fn create_action_server<A, F>(
        &mut self,
        action_name: &str,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer>
    where
        A: Action,
        F: Fn(A::Goal) -> ActionOutcome<A> + Send + Sync + 'static,
    {
        let cb: ActionGoalHandler = Arc::new(move |body| match A::Goal::decode(body) {
            Ok(goal) => {
                let outcome = handler(goal);
                let mut replies = Vec::with_capacity(outcome.feedbacks.len() + 1);
                for fb in outcome.feedbacks {
                    replies.push(("FEEDBACK".into(), fb.encode_to_vec()));
                }
                replies.push(("RESULT".into(), outcome.result.encode_to_vec()));
                replies
            }
            Err(err) => {
                log::warn!("typed action goal decode failed: {err}");
                vec![("RESULT".into(), Vec::new())]
            }
        });
        self.create_action_server_raw(action_name, cb, callback_group)
    }

    /// Register a raw-bytes action server.
    pub fn create_action_server_raw(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        crate::console_topics::check_not_reserved(action_name)?;
        if self.options.is_ws() {
            return Err(grpc_mode_unsupported("create_action_server"));
        }
        let endpoint = self.options.action_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.lock_executor()?.register_action(
            action_name,
            handler,
            group,
            Some(&endpoint),
            None,
        )?;
        Ok(NodeActionServer {
            action_name: action_name.to_string(),
        })
    }

    /// Create a typed action client (ROS 2–style `create_action_client`).
    pub fn create_action_client<A: Action>(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClient<A>> {
        Ok(NodeActionClient {
            inner: self.create_action_client_raw(action_name)?,
            _marker: PhantomData,
        })
    }

    /// Create a raw-bytes action client bound to `action_name`.
    pub fn create_action_client_raw(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClientRaw> {
        self.create_action_client_raw_with_hwm(action_name, self.client_action_hwm())
    }

    /// Like [`create_action_client_raw`](Self::create_action_client_raw), with an explicit HWM.
    pub fn create_action_client_raw_with_hwm(
        &mut self,
        action_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeActionClientRaw> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeActionClientRaw {
                inner: ActionClientInner::Ws(ctx),
                action_name: action_name.into(),
            });
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        Ok(NodeActionClientRaw {
            inner: ActionClientInner::Zmq {
                context: self.context.clone_zmq(),
                endpoint,
                hwm: Mutex::new(hwm),
            },
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
        if self.options.is_ws() {
            return Err(grpc_mode_unsupported(
                "connect_action_client (use create_action_client)",
            ));
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        self.lock_executor()?.connect_action_client(Some(&endpoint))
    }

    /// Submit a goal via the executor (callback receives FEEDBACK / RESULT). Prefer
    /// [`create_action_client`](Self::create_action_client) for a ROS 2–style sync handle.
    pub fn send_goal(
        &mut self,
        action_name: &str,
        body: &[u8],
        callback: ActionMessageCallback,
        goal_id: Option<&str>,
    ) -> Result<String> {
        if self.options.is_ws() {
            return Err(grpc_mode_unsupported(
                "send_goal (use create_action_client)",
            ));
        }
        self.lock_executor()?
            .send_goal(action_name, body, callback, goal_id)
    }

    pub fn cancel_goal(&mut self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        if self.options.is_ws() {
            return Err(grpc_mode_unsupported(
                "cancel_goal (use create_action_client)",
            ));
        }
        self.lock_executor()?
            .cancel_goal(action_name, goal_id, body)
    }

    pub fn shutdown_handle(&mut self) -> Result<ShutdownHandle> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return Ok(self.ensure_ws()?.shutdown_handle());
        }
        self.ensure_executor()?.shutdown_handle()
    }

    pub fn shutdown(&mut self) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            self.ensure_ws()?.shutdown();
            return Ok(());
        }
        self.ensure_executor()?.shutdown()
    }

    /// Spin until [`shutdown`](Self::shutdown) (ROS 2–style `spin(node)`).
    ///
    /// Lazily creates a [`SingleThreadedExecutor`] when none was attached via
    /// `add_node`. Same as `executor.spin()` on the attached / owned executor.
    ///
    /// In gRPC mode, drives subscription callbacks and timers over the gateway.
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
            return Err(grpc_mode_unsupported("start (use spin / spin_once)"));
        }
        self.ensure_executor()?.start()
    }

    pub fn stop(&mut self) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            self.ensure_ws()?.shutdown();
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::runtime::SingleThreadedExecutor;

    #[test]
    fn rejects_reserved_console_names() {
        let mut node = Node::new("pilot");
        assert!(matches!(
            node.create_publisher_raw("/robot_bus/status"),
            Err(BusError::ReservedName { .. })
        ));
        assert!(matches!(
            node.create_service_raw(
                "/robot_bus/topology/register",
                Arc::new(|_| Vec::new()),
                None
            ),
            Err(BusError::ReservedName { .. })
        ));
        assert!(matches!(
            node.create_action_server_raw("/robot_bus/actions", Arc::new(|_| Vec::new()), None),
            Err(BusError::ReservedName { .. })
        ));
    }

    #[test]
    fn node_local_parameters() {
        let mut node = Node::new("pilot");
        node.declare_parameter("max_speed", ParameterValue::Double(1.5))
            .unwrap();
        node.declare_parameter("enabled", ParameterValue::Bool(true))
            .unwrap();
        assert_eq!(
            node.get_parameter("max_speed").unwrap(),
            ParameterValue::Double(1.5)
        );
        node.set_parameter("max_speed", ParameterValue::Double(2.0))
            .unwrap();
        assert!(node.has_parameter("enabled"));
        assert_eq!(node.list_parameters().len(), 2);
        assert!(matches!(
            node.declare_parameter("enabled", ParameterValue::Bool(false)),
            Err(BusError::ParameterAlreadyDeclared { .. })
        ));
        assert!(matches!(
            node.set_parameter("enabled", ParameterValue::Integer(1)),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
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
            node.get_parameter("max_speed").unwrap(),
            ParameterValue::Double(1.25)
        );
        node.load_parameters_from_yaml_str("max_speed: 3.0\n")
            .unwrap();
        assert_eq!(
            node.get_parameter("max_speed").unwrap(),
            ParameterValue::Double(3.0)
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
                    .contains("gRPC")
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
