//! ROS 2–style [`Node`]: named participant with optional executor.
//!
//! Recommended (ROS-like) flow:
//! 1. `let ctx = Context::new();`
//! 2. `let mut node = Node::with_context(&ctx, "pilot");`
//! 3. `node.spin()?;`
//!
//! Convenience: [`Node::new`] creates a private context (fine for tcp/ipc).
//! Same-process **inproc** needs a shared [`crate::Context`] with the embedded
//! broker: `RobotBusBroker::start_with_context` + [`Node::inproc_with_context`].
//!
//! WebSocket RPC client mode (feature `ws`): [`Node::ws`] connects to the
//! broker gateway (subscribe / publish / call service / call action). No ZMQ
//! sockets; service and action **server** APIs return an error.
//!
//! Topic / service / action names are used as given (pass full paths yourself).

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use prost::{Message, Name};

use crate::action_bus::{ActionClient as BusActionClient, ActionKind, ActionMessage};
use crate::errors::{BusError, Result, parse_error_body};
use crate::message_bus::{Publisher as BusPublisher, Subscriber as BusSubscriber};
use crate::runtime::callback_group::{CallbackGroup, CallbackGroupType};
use crate::runtime::console_ready::{self, ReadyKind};
use crate::runtime::context::Context;
use crate::runtime::control_plane::ControlPlaneLedger;
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::executors::{ExecutorHandle, SingleThreadedExecutor};
use crate::runtime::parameters::{ListParametersResult, Parameter, ParameterStore, ParameterValue};
use crate::runtime::qos::QosProfile;
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
use crate::runtime::session::{BrokerSession, ConnectionState, SESSION_CREATE_WAIT};
use crate::runtime::timers::{SubscriptionHandle, TimerCallback, TimerHandle};
use crate::runtime::topic_type_register;
use crate::runtime::topology_register::TopologyEndpointGuard;
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::{WsClientContext, WsRuntime};
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
    /// WebSocket RPC gateway base URL when `transport == "ws"` (e.g. `http://127.0.0.1:15570`).
    pub ws_url: Option<String>,
    /// Embedded console HTTP base URL (same origin as the API listen when co-located).
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

    /// WebSocket RPC gateway (native + browser `/ws`) on the local broker (`http://127.0.0.1:15570`).
    #[cfg(feature = "ws")]
    pub fn ws() -> Self {
        Self::ws_at(WsRuntime::default_url())
    }

    /// WebSocket RPC gateway at `url` (e.g. `http://127.0.0.1:15570`); browsers use `ws(s)://…/ws`.
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
                "ZMQ endpoints are not available in WebSocket RPC node mode".into(),
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
                "message_xsub unset; call wait_for_broker() / NodeOptions::discover() \
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
                "message_xpub unset; call wait_for_broker() / NodeOptions::discover() \
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
                "service_frontend unset; call wait_for_broker() / NodeOptions::discover() \
                 or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn service_backend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.service_backend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "service_backend unset; call wait_for_broker() / NodeOptions::discover() \
                 or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn action_backend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.action_backend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "action_backend unset; call wait_for_broker() / NodeOptions::discover() \
                 or set endpoints explicitly"
                    .into(),
            )),
        }
    }

    pub fn action_frontend_endpoint(&self) -> Result<String> {
        self.require_zmq()?;
        match &self.action_frontend {
            Some(ep) => Ok(ep.clone()),
            None => Err(BusError::Protocol(
                "action_frontend unset; call wait_for_broker() / NodeOptions::discover() \
                 or set endpoints explicitly"
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

fn ws_mode_unsupported(op: &str) -> BusError {
    BusError::Protocol(format!(
        "{op} is not supported in WebSocket RPC node mode (client: subscribe / publish / call service / call action; no servers)"
    ))
}

/// Raw (opaque bytes) publisher from [`Node::create_publisher_raw`].
///
/// ZMQ mode shares one underlying bus PUB socket per node; WebSocket RPC mode issues
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
                "set_high_water_mark is not available for WebSocket RPC publishers".into(),
            )),
        }
    }

    /// Milliseconds; `0` fails immediately when the send HWM is full (drop newest).
    pub fn set_send_timeout_ms(&self, ms: i32) -> Result<()> {
        match &self.backend {
            TopicPublisherBackend::Zmq(inner) => inner.set_send_timeout_ms(ms),
            #[cfg(feature = "ws")]
            TopicPublisherBackend::Ws(_) => Ok(()),
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
    id: u64,
    service_name: String,
}

impl NodeService {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

/// Raw (opaque bytes) service client from [`Node::create_client_raw`].
pub struct NodeServiceClientRaw {
    inner: ServiceClientInner,
    service_name: String,
    console_url: Option<String>,
    _topology: Option<Arc<TopologyEndpointGuard>>,
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

    /// Best-effort: console reports `workers > 0` for this service.
    pub fn service_is_ready(&self) -> bool {
        console_ready::is_ready(
            self.console_url.as_deref(),
            ReadyKind::Service,
            &self.service_name,
        )
    }

    /// Poll until [`service_is_ready`](Self::service_is_ready) or `timeout`.
    pub fn wait_for_service(&self, timeout: Option<Duration>) -> bool {
        console_ready::wait_until_ready(
            self.console_url.as_deref(),
            ReadyKind::Service,
            &self.service_name,
            timeout,
        )
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
                "high_water_mark is not available in WebSocket RPC node mode".into(),
            )),
        }
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        match &self.inner {
            ServiceClientInner::Zmq(client) => client.set_high_water_mark(hwm),
            #[cfg(feature = "ws")]
            ServiceClientInner::Ws(_) => Err(BusError::Protocol(
                "set_high_water_mark is not available in WebSocket RPC node mode".into(),
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

    pub fn service_is_ready(&self) -> bool {
        self.inner.service_is_ready()
    }

    pub fn wait_for_service(&self, timeout: Option<Duration>) -> bool {
        self.inner.wait_for_service(timeout)
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
    id: u64,
    action_name: String,
}

impl NodeActionServer {
    pub fn id(&self) -> u64 {
        self.id
    }

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
    console_url: Option<String>,
    _topology: Option<Arc<TopologyEndpointGuard>>,
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

    /// Best-effort: console reports `workers > 0` for this action.
    pub fn action_server_is_ready(&self) -> bool {
        console_ready::is_ready(
            self.console_url.as_deref(),
            ReadyKind::Action,
            &self.action_name,
        )
    }

    /// Poll until [`action_server_is_ready`](Self::action_server_is_ready) or `timeout`.
    pub fn wait_for_action_server(&self, timeout: Option<Duration>) -> bool {
        console_ready::wait_until_ready(
            self.console_url.as_deref(),
            ReadyKind::Action,
            &self.action_name,
            timeout,
        )
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
                "high_water_mark is not available in WebSocket RPC node mode".into(),
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
                "set_high_water_mark is not available in WebSocket RPC node mode".into(),
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

    pub fn action_server_is_ready(&self) -> bool {
        self.inner.action_server_is_ready()
    }

    pub fn wait_for_action_server(&self, timeout: Option<Duration>) -> bool {
        self.inner.wait_for_action_server(timeout)
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
/// WebSocket RPC mode ([`Node::ws`]): client-only over the broker gateway; does not
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

    /// Create a typed topic publisher (ROS 2 `create_publisher`).
    ///
    /// Uses the node's stream HWM default on first PUB connect. Prefer
    /// [`create_publisher_with_qos`](Self::create_publisher_with_qos) to set KeepLast depth.
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
        self.remember_topic_type(&topic, &M::full_name());
        Ok(pub_)
    }

    /// Create a typed topic publisher with topic QoS (KeepLast depth → HWM).
    ///
    /// Topic reliability is always best-effort.
    pub fn create_publisher_with_qos<M: Message + Name + Default>(
        &mut self,
        topic: impl Into<String>,
        qos: QosProfile,
    ) -> Result<TopicPublisher<M>> {
        let topic = topic.into();
        let pub_ = TopicPublisher {
            inner: self.create_publisher_raw_with_qos(topic.clone(), qos)?,
            _marker: PhantomData,
        };
        self.remember_topic_type(&topic, &M::full_name());
        Ok(pub_)
    }

    /// Create a raw-bytes topic publisher (inherits node stream HWM).
    pub fn create_publisher_raw(&mut self, topic: impl Into<String>) -> Result<TopicPublisherRaw> {
        self.create_publisher_raw_with_hwm(topic, None)
    }

    /// Create a raw-bytes topic publisher with topic QoS.
    pub fn create_publisher_raw_with_qos(
        &mut self,
        topic: impl Into<String>,
        qos: QosProfile,
    ) -> Result<TopicPublisherRaw> {
        self.create_publisher_raw_with_hwm(topic, Some(qos.to_hwm()))
    }

    /// Like [`create_publisher_raw`](Self::create_publisher_raw), optionally setting HWM
    /// on first socket connect. Prefer [`create_publisher_raw_with_qos`] for topic depth.
    pub fn create_publisher_raw_with_hwm(
        &mut self,
        topic: impl Into<String>,
        hwm: Option<HighWaterMark>,
    ) -> Result<TopicPublisherRaw> {
        let topic = topic.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("publisher", &topic));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let _ = hwm; // shared gateway PUB; KeepLast is not per-client on WS publish
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
    /// Prefer [`create_publisher_with_qos`](Self::create_publisher_with_qos) for topic depth.
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
        self.remember_topic_type(&topic, &M::full_name());
        Ok(pub_)
    }

    fn ensure_bus_publisher(&mut self, hwm: Option<HighWaterMark>) -> Result<()> {
        if let Some(pub_) = &self.publisher {
            if let Some(hwm) = hwm {
                pub_.set_high_water_mark(hwm)?;
            }
            return Ok(());
        }
        self.ensure_connected()?;
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
                "set_stream_hwm is not available in WebSocket RPC node mode".into(),
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
                "set_rpc_hwm is not available in WebSocket RPC node mode".into(),
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
                "set_action_hwm is not available in WebSocket RPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_action_hwm(hwm)
    }

    /// Subscribe with a protobuf-typed callback (ROS 2 `create_subscription`).
    ///
    /// Does not change the node's stream HWM. Prefer
    /// [`create_subscription_with_qos`](Self::create_subscription_with_qos) to set
    /// KeepLast depth on the shared SUB socket.
    ///
    /// Decode failures are skipped (logged). `callback_group: None` uses the
    /// node's default mutually exclusive group.
    /// Best-effort registers `topic → M::full_name()` with the broker console.
    pub fn create_subscription<M, F>(
        &mut self,
        topic: &str,
        callback: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle>
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
        let handle = self.create_subscription_raw(topic, cb, Some(&group))?;
        self.remember_topic_type(topic, &M::full_name());
        Ok(handle)
    }

    /// Subscribe with topic QoS (KeepLast depth).
    ///
    /// Topic reliability is always best-effort. On ZMQ, multiple subscriptions on
    /// one node share one SUB socket — the last explicit QoS depth wins for that
    /// socket. On WebSocket, depth sizes that topic's gateway→client queue.
    pub fn create_subscription_with_qos<M, F>(
        &mut self,
        topic: &str,
        qos: QosProfile,
        callback: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle>
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
        let handle = self.create_subscription_raw_with_qos(topic, qos, cb, Some(&group))?;
        self.remember_topic_type(topic, &M::full_name());
        Ok(handle)
    }

    /// Subscribe with a raw-bytes callback (does not change stream HWM).
    pub fn create_subscription_raw(
        &mut self,
        topic: &str,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle> {
        self.create_subscription_raw_inner(topic, None, callback, callback_group)
    }

    /// Subscribe with a raw-bytes callback and topic QoS (KeepLast depth).
    ///
    /// ZMQ: applies to the shared SUB socket HWM. WebSocket: sizes this topic's
    /// gateway→client queue.
    pub fn create_subscription_raw_with_qos(
        &mut self,
        topic: &str,
        qos: QosProfile,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle> {
        self.create_subscription_raw_inner(topic, Some(qos), callback, callback_group)
    }

    fn create_subscription_raw_inner(
        &mut self,
        topic: &str,
        qos: Option<QosProfile>,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle> {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.ensure_connected()?;
        let topology = self.start_topology_guard("subscriber", topic);
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let handle = self.ensure_ws()?.subscribe(topic, callback, group, qos)?;
            self.topology_subscriptions.insert(handle.id(), topology);
            return Ok(handle);
        }
        if let Some(qos) = qos {
            // Apply before connect so the first SUB socket gets the depth; also
            // updates an already-connected shared SUB in place.
            self.set_stream_hwm(qos.to_hwm())?;
        }
        self.ensure_subscriber()?;
        let handle = self.lock_executor()?.subscribe(topic, callback, group)?;
        self.topology_subscriptions.insert(handle.id(), topology);
        Ok(handle)
    }

    /// Destroy a subscription created by [`create_subscription`](Self::create_subscription)
    /// / raw variants. Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_subscription(&mut self, handle: SubscriptionHandle) -> Result<()> {
        self.topology_subscriptions.remove(&handle.id());
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.destroy_subscription(handle);
        }
        self.lock_executor()?.destroy_subscription(handle)
    }

    fn ensure_subscriber(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "internal: ensure_subscriber on gRPC node".into(),
            ));
        }
        self.ensure_connected()?;
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

    /// Alias for [`create_timer`](Self::create_timer) (ROS 2 `create_wall_timer`).
    pub fn create_wall_timer(
        &mut self,
        period: Duration,
        callback: TimerCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<TimerHandle> {
        self.create_timer(period, callback, callback_group)
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

    /// Register a typed service with KeepLast depth → DEALER HWM.
    pub fn create_service_with_qos<S, F>(
        &mut self,
        service_name: &str,
        qos: QosProfile,
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
        self.create_service_raw_with_qos(service_name, qos, cb, callback_group)
    }

    /// Register a raw-bytes service server.
    pub fn create_service_raw(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService> {
        self.create_service_raw_inner(service_name, handler, callback_group, None)
    }

    /// Register a raw-bytes service with KeepLast depth → DEALER HWM.
    pub fn create_service_raw_with_qos(
        &mut self,
        service_name: &str,
        qos: QosProfile,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService> {
        self.create_service_raw_inner(service_name, handler, callback_group, Some(qos.to_hwm()))
    }

    fn create_service_raw_inner(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeService> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("create_service"));
        }
        self.ensure_connected()?;
        let endpoint = self.options.service_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let id = self.lock_executor()?.register_service(
            service_name,
            handler,
            group,
            Some(&endpoint),
            None,
            hwm,
        )?;
        let topology = self.start_topology_guard("service_server", service_name);
        self.topology_services.insert(id, topology);
        Ok(NodeService {
            id,
            service_name: service_name.to_string(),
        })
    }

    /// Destroy a service server created by [`create_service`](Self::create_service).
    /// Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_service(&mut self, handle: &NodeService) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("destroy_service"));
        }
        self.topology_services.remove(&handle.id);
        self.lock_executor()?.destroy_service(handle.id)
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

    /// Create a typed service client with KeepLast depth → DEALER HWM.
    pub fn create_client_with_qos<S: Service>(
        &mut self,
        service_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeServiceClient<S>> {
        Ok(NodeServiceClient {
            inner: self.create_client_raw_with_qos(service_name, qos)?,
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

    /// Create a raw-bytes service client with KeepLast depth → DEALER HWM.
    pub fn create_client_raw_with_qos(
        &mut self,
        service_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeServiceClientRaw> {
        self.create_client_raw_with_hwm(service_name, qos.to_hwm())
    }

    /// Like [`create_client_raw`](Self::create_client_raw), with an explicit HWM.
    pub fn create_client_raw_with_hwm(
        &mut self,
        service_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeServiceClientRaw> {
        let service_name = service_name.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("service_client", &service_name));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeServiceClientRaw {
                inner: ServiceClientInner::Ws(ctx),
                service_name,
                console_url: self.console_url_opt(),
                _topology: topology,
            });
        }
        let endpoint = self.options.service_frontend_endpoint()?;
        Ok(NodeServiceClientRaw {
            inner: ServiceClientInner::Zmq(BusServiceClient::with_context_hwm(
                self.context.zmq(),
                Some(&endpoint),
                hwm,
            )?),
            service_name,
            console_url: self.console_url_opt(),
            _topology: topology,
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

    /// Register a typed action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_with_qos<A, F>(
        &mut self,
        action_name: &str,
        qos: QosProfile,
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
        self.create_action_server_raw_with_qos(action_name, qos, cb, callback_group)
    }

    /// Register a raw-bytes action server.
    pub fn create_action_server_raw(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_inner(action_name, handler, callback_group, None)
    }

    /// Register a raw-bytes action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_raw_with_qos(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_inner(
            action_name,
            handler,
            callback_group,
            Some(qos.to_hwm()),
        )
    }

    fn create_action_server_raw_inner(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeActionServer> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("create_action_server"));
        }
        self.ensure_connected()?;
        let endpoint = self.options.action_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let id = self.lock_executor()?.register_action(
            action_name,
            handler,
            group,
            Some(&endpoint),
            None,
            hwm,
        )?;
        let topology = self.start_topology_guard("action_server", action_name);
        self.topology_actions.insert(id, topology);
        Ok(NodeActionServer {
            id,
            action_name: action_name.to_string(),
        })
    }

    /// Destroy an action server created by [`create_action_server`](Self::create_action_server).
    /// Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_action_server(&mut self, handle: &NodeActionServer) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("destroy_action_server"));
        }
        self.topology_actions.remove(&handle.id);
        self.lock_executor()?.destroy_action_server(handle.id)
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

    /// Create a typed action client with KeepLast depth → DEALER HWM.
    pub fn create_action_client_with_qos<A: Action>(
        &mut self,
        action_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeActionClient<A>> {
        Ok(NodeActionClient {
            inner: self.create_action_client_raw_with_qos(action_name, qos)?,
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

    /// Create a raw-bytes action client with KeepLast depth → DEALER HWM.
    pub fn create_action_client_raw_with_qos(
        &mut self,
        action_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeActionClientRaw> {
        self.create_action_client_raw_with_hwm(action_name, qos.to_hwm())
    }

    /// Like [`create_action_client_raw`](Self::create_action_client_raw), with an explicit HWM.
    pub fn create_action_client_raw_with_hwm(
        &mut self,
        action_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeActionClientRaw> {
        let action_name = action_name.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("action_client", &action_name));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeActionClientRaw {
                inner: ActionClientInner::Ws(ctx),
                action_name,
                console_url: self.console_url_opt(),
                _topology: topology,
            });
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        Ok(NodeActionClientRaw {
            inner: ActionClientInner::Zmq {
                context: self.context.clone_zmq(),
                endpoint,
                hwm: Mutex::new(hwm),
            },
            action_name,
            console_url: self.console_url_opt(),
            _topology: topology,
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

    fn start_topology_guard(&self, kind: &str, name: &str) -> Arc<TopologyEndpointGuard> {
        let guard = TopologyEndpointGuard::start(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &self.name,
            kind,
            name,
        );
        self.control_plane.remember_topology(&guard);
        guard
    }

    /// Connect the executor-owned action client used by callback-style [`send_goal`](Self::send_goal).
    pub fn connect_action_client(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported(
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
            return Err(ws_mode_unsupported("send_goal (use create_action_client)"));
        }
        self.lock_executor()?
            .send_goal(action_name, body, callback, goal_id)
    }

    pub fn cancel_goal(&mut self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported(
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
