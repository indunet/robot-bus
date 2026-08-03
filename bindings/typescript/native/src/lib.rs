//! napi-rs bindings mirroring the Python PyO3 surface (`src/python_api.rs`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{
    ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi_derive::napi;
use robot_bus::action_bus::ActionKind;
use robot_bus::broker::{
    apply_federation_opts, parse_robot_bus_config, robot_bus_broker_help,
    RobotBusBroker as RustRobotBusBroker, RobotBusConfig,
};
use robot_bus::discovery::{wait as discover_wait, DiscoverOpts as RustDiscoverOpts};
use robot_bus::errors::BusError;
use robot_bus::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use robot_bus::runtime::{
    ActionGoalHandler, CallbackGroup, CallbackGroupType, Context as RustContext,
    MultiThreadedExecutor as RustMultiThreadedExecutor, Node as RustNode,
    NodeActionClientRaw as RustNodeActionClient, NodeOptions as RustNodeOptions,
    NodeServiceClientRaw as RustNodeServiceClient, ParameterValue, ServiceHandler,
    ShutdownHandle as RustShutdownHandle, SingleThreadedExecutor as RustSingleThreadedExecutor,
    TimerCallback, TimerHandle as RustTimerHandle, TopicPublisherRaw as RustTopicPublisher,
};
use robot_bus::tf::{Buffer as RustTfBuffer, SharedBuffer, TfListener as RustTfListener};
use robot_bus::tf2_msgs::msg::v1::TfMessage;
use robot_bus::{shutdown, transports};
use prost::Message;

fn bus_err(err: BusError) -> Error {
    Error::from_reason(err.to_string())
}

fn anyhow_err(err: anyhow::Error) -> Error {
    Error::from_reason(err.to_string())
}

fn map_endpoint_err(err: String) -> Error {
    Error::from_reason(err)
}

fn parameter_value_from_js(value: Unknown) -> Result<ParameterValue> {
    match value.get_type()? {
        ValueType::Boolean => Ok(ParameterValue::Bool(value.coerce_to_bool()?.get_value()?)),
        ValueType::Number => {
            let n = value.coerce_to_number()?.get_double()?;
            if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                Ok(ParameterValue::Integer(n as i64))
            } else {
                Ok(ParameterValue::Double(n))
            }
        }
        ValueType::String => Ok(ParameterValue::String(
            value.coerce_to_string()?.into_utf8()?.into_owned()?,
        )),
        other => Err(Error::from_reason(format!(
            "parameter value must be bool, number, or string; got {other:?}"
        ))),
    }
}

fn parameter_value_to_js(env: &Env, value: ParameterValue) -> Result<Unknown> {
    Ok(match value {
        ParameterValue::Bool(v) => env.get_boolean(v)?.into_unknown(),
        ParameterValue::Integer(v) => env.create_int64(v)?.into_unknown(),
        ParameterValue::Double(v) => env.create_double(v)?.into_unknown(),
        ParameterValue::String(v) => env.create_string(&v)?.into_unknown(),
    })
}

fn node_options(
    host: &str,
    transport: &str,
    grpc_url: Option<String>,
    message_xsub: Option<String>,
    message_xpub: Option<String>,
    service_frontend: Option<String>,
    service_backend: Option<String>,
    action_backend: Option<String>,
    action_frontend: Option<String>,
) -> Result<RustNodeOptions> {
    if transport == "grpc" {
        return Ok(match grpc_url {
            Some(url) => RustNodeOptions::grpc_at(url),
            None => RustNodeOptions::grpc(),
        });
    }
    if grpc_url.is_some() {
        return Err(Error::from_reason(
            "grpc_url is only valid when transport=\"grpc\"",
        ));
    }
    Ok(RustNodeOptions {
        host: host.into(),
        transport: transport.into(),
        grpc_url: None,
        console_url: None,
        message_xsub,
        message_xpub,
        service_frontend,
        service_backend,
        action_backend,
        action_frontend,
    })
}

fn normalize_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
}

fn action_kind_str(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Goal => "GOAL",
        ActionKind::Feedback => "FEEDBACK",
        ActionKind::Result => "RESULT",
        ActionKind::Cancel => "CANCEL",
    }
}

#[napi]
pub fn message_xsub_endpoint(
    host: Option<String>,
    transport: Option<String>,
) -> Result<String> {
    let host = host.unwrap_or_else(|| "localhost".into());
    let transport = transport.unwrap_or_else(|| "tcp".into());
    transports::message_xsub_endpoint(&host, &transport).map_err(map_endpoint_err)
}

#[napi]
pub fn message_xpub_endpoint(
    host: Option<String>,
    transport: Option<String>,
) -> Result<String> {
    let host = host.unwrap_or_else(|| "localhost".into());
    let transport = transport.unwrap_or_else(|| "tcp".into());
    transports::message_xpub_endpoint(&host, &transport).map_err(map_endpoint_err)
}

#[napi]
#[derive(Clone)]
pub struct ShutdownHandle {
    inner: RustShutdownHandle,
}

#[napi]
impl ShutdownHandle {
    #[napi]
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }

    #[napi]
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }
}

#[napi]
pub struct TimerHandle {
    inner: RustTimerHandle,
}

#[napi]
pub enum JsCallbackGroupType {
    MutuallyExclusive = 0,
    Reentrant = 1,
}

impl From<JsCallbackGroupType> for CallbackGroupType {
    fn from(value: JsCallbackGroupType) -> Self {
        match value {
            JsCallbackGroupType::MutuallyExclusive => CallbackGroupType::MutuallyExclusive,
            JsCallbackGroupType::Reentrant => CallbackGroupType::Reentrant,
        }
    }
}

#[napi]
#[derive(Clone)]
pub struct JsCallbackGroup {
    inner: CallbackGroup,
}

#[napi]
impl JsCallbackGroup {
    #[napi(getter)]
    pub fn id(&self) -> u64 {
        self.inner.id()
    }

    #[napi(getter)]
    pub fn kind(&self) -> JsCallbackGroupType {
        match self.inner.kind() {
            CallbackGroupType::MutuallyExclusive => JsCallbackGroupType::MutuallyExclusive,
            CallbackGroupType::Reentrant => JsCallbackGroupType::Reentrant,
        }
    }
}

#[napi]
pub struct Publisher {
    inner: RustPublisher,
}

#[napi]
impl Publisher {
    #[napi(constructor)]
    pub fn new(endpoint: Option<String>) -> Result<Self> {
        Ok(Self {
            inner: RustPublisher::new(endpoint.as_deref()).map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn publish(&self, topic: String, payload: Buffer) -> Result<()> {
        self.inner.publish(&topic, &payload).map_err(bus_err)
    }

    #[napi(getter)]
    pub fn endpoint(&self) -> String {
        self.inner.endpoint().to_string()
    }
}

#[napi]
pub struct Subscriber {
    inner: RustSubscriber,
}

#[napi]
impl Subscriber {
    #[napi(constructor)]
    pub fn new(endpoint: Option<String>) -> Result<Self> {
        Ok(Self {
            inner: RustSubscriber::new(endpoint.as_deref()).map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn subscribe(&self, topic: String) -> Result<()> {
        self.inner.subscribe(&topic).map_err(bus_err)
    }

    #[napi]
    pub fn unsubscribe(&self, topic: String) -> Result<()> {
        self.inner.unsubscribe(&topic).map_err(bus_err)
    }

    /// Return `{ topic, payload }`. `timeout` is seconds; omit to block forever.
    #[napi]
    pub fn receive(&self, timeout: Option<f64>) -> Result<TopicMessage> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let (topic, payload) = self.inner.receive(timeout).map_err(bus_err)?;
        Ok(TopicMessage {
            topic,
            payload: Buffer::from(payload),
        })
    }

    #[napi(getter)]
    pub fn endpoint(&self) -> String {
        self.inner.endpoint().to_string()
    }
}

#[napi(object)]
pub struct TopicMessage {
    pub topic: String,
    pub payload: Buffer,
}

#[napi(object)]
pub struct ActionEvent {
    pub kind: String,
    pub body: Buffer,
    pub goal_id: String,
    pub action_name: String,
}

#[napi(object)]
pub struct ActionPhaseReply {
    pub phase: String,
    pub body: Buffer,
}

#[napi]
pub struct TopicPublisher {
    inner: RustTopicPublisher,
}

#[napi]
impl TopicPublisher {
    #[napi(getter)]
    pub fn topic(&self) -> String {
        self.inner.topic().to_string()
    }

    #[napi]
    pub fn publish(&self, payload: Buffer) -> Result<()> {
        self.inner.publish(&payload).map_err(bus_err)
    }
}

#[napi]
pub struct ServiceClient {
    inner: RustNodeServiceClient,
}

#[napi]
impl ServiceClient {
    #[napi(getter)]
    pub fn service_name(&self) -> String {
        self.inner.service_name().to_string()
    }

    /// Call the bound service. `timeout` is seconds; omit to wait indefinitely.
    #[napi]
    pub fn call(&self, body: Buffer, timeout: Option<f64>) -> Result<Buffer> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let reply = self.inner.call(&body, timeout).map_err(bus_err)?;
        Ok(Buffer::from(reply))
    }
}

#[napi]
pub struct ActionClient {
    inner: RustNodeActionClient,
}

#[napi]
impl ActionClient {
    #[napi(getter)]
    pub fn action_name(&self) -> String {
        self.inner.action_name().to_string()
    }

    #[napi]
    pub fn send_goal(
        &self,
        body: Buffer,
        goal_id: Option<String>,
        timeout: Option<f64>,
    ) -> Result<Vec<ActionEvent>> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let messages = self
            .inner
            .send_goal(&body, goal_id.as_deref(), timeout)
            .map_err(bus_err)?;
        Ok(messages
            .into_iter()
            .map(|msg| ActionEvent {
                kind: action_kind_str(msg.kind).to_string(),
                body: Buffer::from(msg.body),
                goal_id: msg.goal_id,
                action_name: msg.action_name,
            })
            .collect())
    }

    #[napi]
    pub fn cancel(
        &self,
        goal_id: String,
        body: Option<Buffer>,
        timeout: Option<f64>,
    ) -> Result<ActionEvent> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let body = body.as_deref().map(|b| b.as_ref()).unwrap_or(b"");
        let msg = self
            .inner
            .cancel(&goal_id, body, timeout)
            .map_err(bus_err)?;
        Ok(ActionEvent {
            kind: action_kind_str(msg.kind).to_string(),
            body: Buffer::from(msg.body),
            goal_id: msg.goal_id,
            action_name: msg.action_name,
        })
    }
}

type MsgTsfn = ThreadsafeFunction<(String, Vec<u8>), ErrorStrategy::Fatal>;
type VoidTsfn = ThreadsafeFunction<(), ErrorStrategy::Fatal>;
type ServiceTsfn = ThreadsafeFunction<Vec<u8>, ErrorStrategy::CalleeHandled>;
type ActionTsfn = ThreadsafeFunction<Vec<u8>, ErrorStrategy::CalleeHandled>;

/// Shared ZeroMQ runtime context (required for same-process inproc).
#[napi]
pub struct Context {
    inner: RustContext,
}

#[napi]
impl Context {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RustContext::new(),
        }
    }
}

#[napi]
pub struct Node {
    inner: RustNode,
}

#[napi]
impl Node {
    #[napi(constructor)]
    pub fn new(
        name: String,
        host: Option<String>,
        transport: Option<String>,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Result<Self> {
        let host = host.unwrap_or_else(|| "localhost".into());
        let transport = transport.unwrap_or_else(|| "tcp".into());
        Ok(Self {
            inner: RustNode::with_options(
                name,
                node_options(
                    &host,
                    &transport,
                    grpc_url,
                    message_xsub,
                    message_xpub,
                    service_frontend,
                    service_backend,
                    action_backend,
                    action_frontend,
                )?,
            ),
        })
    }

    #[napi(factory)]
    pub fn tcp(name: String, host: Option<String>) -> Self {
        let host = host.unwrap_or_else(|| "localhost".into());
        Self {
            inner: RustNode::with_options(name, RustNodeOptions::tcp_at(&host)),
        }
    }

    #[napi(factory)]
    pub fn ipc(name: String, path: Option<String>) -> Self {
        let options = match path.as_deref() {
            Some(dir) => RustNodeOptions::ipc_at(dir),
            None => RustNodeOptions::ipc(),
        };
        Self {
            inner: RustNode::with_options(name, options),
        }
    }

    #[napi(factory)]
    pub fn inproc(name: String, prefix: Option<String>) -> Self {
        let options = match prefix.as_deref() {
            Some(p) => RustNodeOptions::inproc_at(p),
            None => RustNodeOptions::inproc(),
        };
        Self {
            inner: RustNode::with_options(name, options),
        }
    }

    /// Same-process inproc sharing `context` with an embedded broker.
    #[napi(factory)]
    pub fn inproc_with_context(
        context: &Context,
        name: String,
        prefix: Option<String>,
    ) -> Self {
        match prefix.as_deref() {
            Some(p) => Self {
                inner: RustNode::inproc_at_with_context(context.inner.clone(), name, p),
            },
            None => Self {
                inner: RustNode::inproc_with_context(context.inner.clone(), name),
            },
        }
    }

    #[napi(factory)]
    pub fn with_context(
        context: &Context,
        name: String,
        host: Option<String>,
        transport: Option<String>,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Result<Self> {
        let host = host.unwrap_or_else(|| "localhost".into());
        let transport = transport.unwrap_or_else(|| "tcp".into());
        Ok(Self {
            inner: RustNode::with_context(
                context.inner.clone(),
                name,
                node_options(
                    &host,
                    &transport,
                    grpc_url,
                    message_xsub,
                    message_xpub,
                    service_frontend,
                    service_backend,
                    action_backend,
                    action_frontend,
                )?,
            ),
        })
    }

    #[napi(factory)]
    pub fn grpc(name: String) -> Self {
        Self {
            inner: RustNode::grpc(name),
        }
    }

    #[napi(factory)]
    pub fn grpc_at(name: String, url: String) -> Self {
        Self {
            inner: RustNode::grpc_at(name, url),
        }
    }

    /// Discover a broker via UDP multicast, then connect with `transport`.
    #[napi(factory)]
    pub fn discover(name: String, options: Option<DiscoverNodeOptions>) -> Result<Self> {
        let o = options.unwrap_or_default();
        let transport = o.transport.unwrap_or_else(|| "tcp".into());
        let base = match transport.as_str() {
            "tcp" => RustNodeOptions::tcp(),
            "ipc" => RustNodeOptions::ipc(),
            "inproc" => RustNodeOptions::inproc(),
            "grpc" => RustNodeOptions::grpc(),
            other => {
                return Err(Error::from_reason(format!("unknown transport {other:?}")));
            }
        };
        let mut opts = RustDiscoverOpts {
            domain_id: o.domain_id.unwrap_or(0),
            broker_id: o.broker_id.filter(|s| !s.is_empty()),
            ..Default::default()
        };
        if let Some(addr) = o.multicast_addr.as_deref() {
            if !addr.is_empty() {
                opts.multicast_addr = addr
                    .parse()
                    .map_err(|e| Error::from_reason(format!("invalid multicast_addr: {e}")))?;
            }
        }
        if let Some(port) = o.multicast_port {
            if port != 0 {
                opts.multicast_port = port as u16;
            }
        }
        if let Some(timeout) = o.timeout_secs {
            if timeout > 0.0 {
                opts.timeout = Duration::from_secs_f64(timeout);
            }
        }
        let applied = discover_wait(opts)
            .and_then(|ann| ann.apply(base))
            .map_err(bus_err)?;
        Ok(Self {
            inner: RustNode::with_options(name, applied),
        })
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi]
    pub fn declare_parameter(&mut self, name: String, value: Unknown) -> Result<()> {
        self.inner
            .declare_parameter(name, parameter_value_from_js(value)?)
            .map_err(bus_err)
    }

    #[napi]
    pub fn get_parameter(&self, env: Env, name: String) -> Result<Unknown> {
        let value = self.inner.get_parameter(&name).map_err(bus_err)?;
        parameter_value_to_js(&env, value)
    }

    #[napi]
    pub fn set_parameter(&mut self, name: String, value: Unknown) -> Result<()> {
        self.inner
            .set_parameter(&name, parameter_value_from_js(value)?)
            .map_err(bus_err)
    }

    #[napi]
    pub fn has_parameter(&self, name: String) -> bool {
        self.inner.has_parameter(&name)
    }

    #[napi]
    pub fn list_parameters(&self, env: Env) -> Result<Vec<Unknown>> {
        let mut out = Vec::new();
        for param in self.inner.list_parameters() {
            let mut obj = env.create_object()?;
            obj.set_named_property("name", env.create_string(&param.name)?)?;
            obj.set_named_property("value", parameter_value_to_js(&env, param.value)?)?;
            out.push(obj.into_unknown());
        }
        Ok(out)
    }

    #[napi]
    pub fn load_parameters_from_yaml_str(&mut self, yaml: String) -> Result<()> {
        self.inner
            .load_parameters_from_yaml_str(&yaml)
            .map_err(bus_err)
    }

    #[napi]
    pub fn load_parameters_from_yaml_file(&mut self, path: String) -> Result<()> {
        self.inner
            .load_parameters_from_yaml_file(&path)
            .map_err(bus_err)
    }

    #[napi]
    pub fn create_callback_group(&self, kind: JsCallbackGroupType) -> JsCallbackGroup {
        JsCallbackGroup {
            inner: self.inner.create_callback_group(kind.into()),
        }
    }

    #[napi]
    pub fn create_publisher(&mut self, topic: String) -> Result<TopicPublisher> {
        Ok(TopicPublisher {
            inner: self.inner.create_publisher_raw(&topic).map_err(bus_err)?,
        })
    }

    /// Register a subscription callback `(topic: string, payload: Buffer) => void`.
    #[napi]
    pub fn create_subscription(
        &mut self,
        topic: String,
        callback: JsFunction,
        callback_group: Option<&JsCallbackGroup>,
    ) -> Result<()> {
        let tsfn: MsgTsfn = callback.create_threadsafe_function(0, |ctx| {
            let (topic, payload) = ctx.value;
            Ok(vec![
                ctx.env.create_string_from_std(topic)?.into_unknown(),
                ctx.env.create_buffer_with_data(payload)?.into_unknown(),
            ])
        })?;
        let tsfn = Arc::new(tsfn);
        let cb: robot_bus::runtime::MessageCallback = Arc::new(move |topic, payload| {
            let _ = tsfn.call(
                (topic.to_string(), payload.to_vec()),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        });
        let group = callback_group.map(|g| &g.inner);
        self.inner
            .create_subscription_raw(&topic, cb, group)
            .map_err(bus_err)
    }

    /// Periodic timer; `callback()` takes no arguments. `period` is seconds.
    #[napi]
    pub fn create_timer(
        &mut self,
        period: f64,
        callback: JsFunction,
        callback_group: Option<&JsCallbackGroup>,
    ) -> Result<TimerHandle> {
        let tsfn: VoidTsfn =
            callback.create_threadsafe_function(0, |_ctx| Ok(Vec::<Unknown>::new()))?;
        let tsfn = Arc::new(tsfn);
        let cb: TimerCallback = Arc::new(move || {
            let _ = tsfn.call((), ThreadsafeFunctionCallMode::NonBlocking);
        });
        let group = callback_group.map(|g| &g.inner);
        let handle = self
            .inner
            .create_timer(Duration::from_secs_f64(period), cb, group)
            .map_err(bus_err)?;
        Ok(TimerHandle { inner: handle })
    }

    #[napi]
    pub fn cancel_timer(&mut self, handle: &TimerHandle) -> Result<()> {
        self.inner.cancel_timer(handle.inner).map_err(bus_err)
    }

    /// Register a service server. `handler(body: Buffer) => Buffer`.
    #[napi]
    pub fn create_service(
        &mut self,
        service_name: String,
        handler: JsFunction,
        callback_group: Option<&JsCallbackGroup>,
    ) -> Result<()> {
        let tsfn: ServiceTsfn = handler.create_threadsafe_function(0, |ctx| {
            Ok(vec![Buffer::from(ctx.value)])
        })?;
        let tsfn = Arc::new(tsfn);
        let cb: ServiceHandler = Arc::new(move |body| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
            let _ = tsfn.call_with_return_value(
                Ok(body.to_vec()),
                ThreadsafeFunctionCallMode::Blocking,
                move |ret: Buffer| {
                    let _ = tx.send(ret.to_vec());
                    Ok(())
                },
            );
            rx.recv_timeout(Duration::from_secs(30))
                .unwrap_or_default()
        });
        let group = callback_group.map(|g| &g.inner);
        self.inner
            .create_service_raw(&service_name, cb, group)
            .map(|_| ())
            .map_err(bus_err)
    }

    #[napi]
    pub fn create_client(&mut self, service_name: String) -> Result<ServiceClient> {
        Ok(ServiceClient {
            inner: self
                .inner
                .create_client_raw(&service_name)
                .map_err(bus_err)?,
        })
    }

    /// Register an action server.
    /// `handler(payload: Buffer) => Array<{ phase: string, body: Buffer }>`.
    #[napi]
    pub fn create_action_server(
        &mut self,
        action_name: String,
        handler: JsFunction,
        callback_group: Option<&JsCallbackGroup>,
    ) -> Result<()> {
        let tsfn: ActionTsfn = handler.create_threadsafe_function(0, |ctx| {
            Ok(vec![Buffer::from(ctx.value)])
        })?;
        let tsfn = Arc::new(tsfn);
        let cb: ActionGoalHandler = Arc::new(move |payload| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<(String, Vec<u8>)>>(1);
            let _ = tsfn.call_with_return_value(
                Ok(payload.to_vec()),
                ThreadsafeFunctionCallMode::Blocking,
                move |ret: Vec<ActionPhaseReply>| {
                    let replies = ret
                        .into_iter()
                        .map(|item| (item.phase, item.body.to_vec()))
                        .collect();
                    let _ = tx.send(replies);
                    Ok(())
                },
            );
            rx.recv_timeout(Duration::from_secs(30))
                .unwrap_or_default()
        });
        let group = callback_group.map(|g| &g.inner);
        self.inner
            .create_action_server_raw(&action_name, cb, group)
            .map(|_| ())
            .map_err(bus_err)
    }

    #[napi]
    pub fn create_action_client(&mut self, action_name: String) -> Result<ActionClient> {
        Ok(ActionClient {
            inner: self
                .inner
                .create_action_client_raw(&action_name)
                .map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn connect_action_client(&mut self) -> Result<()> {
        self.inner.connect_action_client().map_err(bus_err)
    }

    #[napi]
    pub fn shutdown_handle(&mut self) -> Result<ShutdownHandle> {
        Ok(ShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[napi]
    pub fn spin_once(&mut self, timeout: Option<f64>) -> Result<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    #[napi]
    pub fn spin(&mut self) -> Result<()> {
        self.inner.spin().map_err(bus_err)
    }

    #[napi]
    pub fn start(&mut self) -> Result<()> {
        self.inner.start().map_err(bus_err)
    }

    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        self.inner.stop().map_err(bus_err)
    }

    #[napi]
    pub fn wait(&mut self) -> Result<()> {
        self.inner.wait().map_err(bus_err)
    }
}

#[napi]
pub struct SingleThreadedExecutor {
    inner: RustSingleThreadedExecutor,
}

#[napi]
impl SingleThreadedExecutor {
    #[napi(constructor)]
    pub fn new(context: Option<&Context>) -> Self {
        Self {
            inner: match context {
                Some(c) => RustSingleThreadedExecutor::with_context(c.inner.clone()),
                None => RustSingleThreadedExecutor::new(),
            },
        }
    }

    #[napi]
    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[napi]
    pub fn create_node(
        &self,
        name: String,
        host: Option<String>,
        transport: Option<String>,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Result<Node> {
        let host = host.unwrap_or_else(|| "localhost".into());
        let transport = transport.unwrap_or_else(|| "tcp".into());
        let options = node_options(
            &host,
            &transport,
            grpc_url,
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        )?;
        Ok(Node {
            inner: self
                .inner
                .create_node_with_options(name, options)
                .map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        Ok(ShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[napi]
    pub fn spin_once(&self, timeout: Option<f64>) -> Result<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    #[napi]
    pub fn spin(&self) -> Result<()> {
        self.inner.spin().map_err(bus_err)
    }

    #[napi]
    pub fn start(&self) -> Result<()> {
        self.inner.start().map_err(bus_err)
    }

    #[napi]
    pub fn stop(&self) -> Result<()> {
        self.inner.stop().map_err(bus_err)
    }

    #[napi]
    pub fn wait(&self) -> Result<()> {
        self.inner.wait().map_err(bus_err)
    }
}

#[napi]
pub struct MultiThreadedExecutor {
    inner: RustMultiThreadedExecutor,
}

#[napi]
impl MultiThreadedExecutor {
    #[napi(constructor)]
    pub fn new(num_threads: Option<u32>, context: Option<&Context>) -> Self {
        let n = num_threads.unwrap_or(4) as usize;
        Self {
            inner: match context {
                Some(c) => RustMultiThreadedExecutor::with_context(c.inner.clone(), n),
                None => RustMultiThreadedExecutor::new(n),
            },
        }
    }

    #[napi]
    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[napi]
    pub fn create_node(
        &self,
        name: String,
        host: Option<String>,
        transport: Option<String>,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Result<Node> {
        let host = host.unwrap_or_else(|| "localhost".into());
        let transport = transport.unwrap_or_else(|| "tcp".into());
        let options = node_options(
            &host,
            &transport,
            grpc_url,
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        )?;
        Ok(Node {
            inner: self
                .inner
                .create_node_with_options(name, options)
                .map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        Ok(ShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[napi]
    pub fn spin_once(&self, timeout: Option<f64>) -> Result<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    #[napi]
    pub fn spin(&self) -> Result<()> {
        self.inner.spin().map_err(bus_err)
    }
}

#[napi(object)]
#[derive(Default)]
pub struct DiscoverNodeOptions {
    pub transport: Option<String>,
    pub domain_id: Option<u32>,
    pub broker_id: Option<String>,
    pub multicast_addr: Option<String>,
    pub multicast_port: Option<u32>,
    pub timeout_secs: Option<f64>,
}

#[napi(object)]
pub struct BrokerStartOptions {
    pub message_xsub_bind: Option<String>,
    pub message_xpub_bind: Option<String>,
    pub message_snd_hwm: Option<i32>,
    pub message_rcv_hwm: Option<i32>,
    pub service_frontend_bind: Option<String>,
    pub service_backend_bind: Option<String>,
    pub service_snd_hwm: Option<i32>,
    pub service_rcv_hwm: Option<i32>,
    pub service_heartbeat_interval_ms: Option<u32>,
    pub service_heartbeat_timeout_ms: Option<u32>,
    pub action_frontend_bind: Option<String>,
    pub action_backend_bind: Option<String>,
    pub action_snd_hwm: Option<i32>,
    pub action_rcv_hwm: Option<i32>,
    pub action_heartbeat_interval_ms: Option<u32>,
    pub action_heartbeat_timeout_ms: Option<u32>,
    pub action_pending_timeout_ms: Option<u32>,
    pub snd_hwm: Option<i32>,
    pub rcv_hwm: Option<i32>,
    pub heartbeat_interval_ms: Option<u32>,
    pub heartbeat_timeout_ms: Option<u32>,
    pub tcp_only: Option<bool>,
    pub grpc_listen: Option<String>,
    pub cors_origins: Option<Vec<String>>,
    pub console_listen: Option<String>,
    pub no_console: Option<bool>,
    pub broker_id: Option<String>,
    pub message_peers: Option<Vec<String>>,
    pub service_peers: Option<Vec<String>>,
    pub action_peers: Option<Vec<String>>,
    pub domain_id: Option<u32>,
    pub no_discovery: Option<bool>,
    pub advertise_host: Option<String>,
    pub discovery_addr: Option<String>,
    pub discovery_port: Option<u32>,
}

#[napi]
pub struct RobotBusBroker {
    inner: Option<RustRobotBusBroker>,
}

#[napi]
impl RobotBusBroker {
    #[napi(factory)]
    pub fn start(
        options: Option<BrokerStartOptions>,
        context: Option<&Context>,
    ) -> Result<Self> {
        let mut config = RobotBusConfig::default();
        if let Some(o) = options {
            if let Some(v) = o.message_xsub_bind {
                config.message.xsub_bind = normalize_bind(&v);
            }
            if let Some(v) = o.message_xpub_bind {
                config.message.xpub_bind = normalize_bind(&v);
            }
            if let Some(v) = o.message_snd_hwm {
                config.message.snd_hwm = v;
            }
            if let Some(v) = o.message_rcv_hwm {
                config.message.rcv_hwm = v;
            }
            if let Some(v) = o.service_frontend_bind {
                config.service.frontend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.service_backend_bind {
                config.service.backend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.service_snd_hwm {
                config.service.snd_hwm = v;
            }
            if let Some(v) = o.service_rcv_hwm {
                config.service.rcv_hwm = v;
            }
            if let Some(v) = o.service_heartbeat_interval_ms {
                config.service.heartbeat_interval_ms = v as u64;
            }
            if let Some(v) = o.service_heartbeat_timeout_ms {
                config.service.heartbeat_timeout_ms = v as u64;
            }
            if let Some(v) = o.action_frontend_bind {
                config.action.frontend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.action_backend_bind {
                config.action.backend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.action_snd_hwm {
                config.action.snd_hwm = v;
            }
            if let Some(v) = o.action_rcv_hwm {
                config.action.rcv_hwm = v;
            }
            if let Some(v) = o.action_heartbeat_interval_ms {
                config.action.heartbeat_interval_ms = v as u64;
            }
            if let Some(v) = o.action_heartbeat_timeout_ms {
                config.action.heartbeat_timeout_ms = v as u64;
            }
            if let Some(v) = o.action_pending_timeout_ms {
                config.action.pending_timeout_ms = v as u64;
            }
            if let Some(v) = o.snd_hwm {
                config.message.snd_hwm = v;
                config.service.snd_hwm = v;
                config.action.snd_hwm = v;
            }
            if let Some(v) = o.rcv_hwm {
                config.message.rcv_hwm = v;
                config.service.rcv_hwm = v;
                config.action.rcv_hwm = v;
            }
            if let Some(v) = o.heartbeat_interval_ms {
                config.service.heartbeat_interval_ms = v as u64;
                config.action.heartbeat_interval_ms = v as u64;
            }
            if let Some(v) = o.heartbeat_timeout_ms {
                config.service.heartbeat_timeout_ms = v as u64;
                config.action.heartbeat_timeout_ms = v as u64;
            }
            if o.tcp_only.unwrap_or(false) {
                config.message.bind_all_transports = false;
                config.service.bind_all_transports = false;
                config.action.bind_all_transports = false;
            }
            if let Some(v) = o.grpc_listen {
                config.grpc.listen = v
                    .parse()
                    .map_err(|e| Error::from_reason(format!("invalid grpc_listen: {e}")))?;
            }
            if let Some(v) = o.cors_origins {
                config.grpc.cors_origins = v;
            }
            if o.no_console.unwrap_or(false) {
                config.console.enabled = false;
            }
            if let Some(v) = o.console_listen {
                config.console.listen = v
                    .parse()
                    .map_err(|e| Error::from_reason(format!("invalid console_listen: {e}")))?;
                config.console.enabled = true;
            }
            apply_federation_opts(
                &mut config,
                o.broker_id.as_deref(),
                o.message_peers.as_deref().unwrap_or(&[]),
                o.service_peers.as_deref().unwrap_or(&[]),
                o.action_peers.as_deref().unwrap_or(&[]),
            )
            .map_err(anyhow_err)?;
            if o.no_discovery.unwrap_or(false) {
                config.discovery.enabled = false;
            }
            if let Some(v) = o.domain_id {
                config.discovery.domain_id = v;
            }
            if let Some(v) = o.advertise_host {
                if !v.is_empty() {
                    config.discovery.advertise_host = Some(v);
                }
            }
            if let Some(v) = o.discovery_addr {
                if !v.is_empty() {
                    config.discovery.multicast_addr = v.parse().map_err(|e| {
                        Error::from_reason(format!("invalid discovery_addr: {e}"))
                    })?;
                }
            }
            if let Some(port) = o.discovery_port {
                if port != 0 {
                    config.discovery.multicast_port = port as u16;
                }
            }
        }

        let broker = match context {
            Some(c) => RustRobotBusBroker::start_with_context(c.inner.clone(), config),
            None => RustRobotBusBroker::start(config),
        }
        .map_err(anyhow_err)?;
        Ok(Self {
            inner: Some(broker),
        })
    }

    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        if let Some(broker) = self.inner.take() {
            broker.stop().map_err(anyhow_err)?;
        }
        Ok(())
    }

    fn with_broker<T>(&self, f: impl FnOnce(&RustRobotBusBroker) -> T) -> Result<T> {
        self.inner
            .as_ref()
            .map(f)
            .ok_or_else(|| Error::from_reason("broker already stopped"))
    }

    #[napi(getter)]
    pub fn message_xsub_bind(&self) -> Result<String> {
        self.with_broker(|b| b.message.xsub_bind.clone())
    }

    #[napi(getter)]
    pub fn message_xpub_bind(&self) -> Result<String> {
        self.with_broker(|b| b.message.xpub_bind.clone())
    }

    #[napi(getter)]
    pub fn service_frontend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.service.frontend_bind.clone())
    }

    #[napi(getter)]
    pub fn service_backend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.service.backend_bind.clone())
    }

    #[napi(getter)]
    pub fn action_frontend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.action.frontend_bind.clone())
    }

    #[napi(getter)]
    pub fn action_backend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.action.backend_bind.clone())
    }

    #[napi(getter)]
    pub fn grpc_listen(&self) -> Result<String> {
        self.with_broker(|b| b.grpc_listen().to_string())
    }

    #[napi(getter)]
    pub fn console_listen(&self) -> Result<Option<String>> {
        self.with_broker(|b| b.console_listen().map(|a| a.to_string()))
    }
}

/// Blocking CLI entry: start broker and wait for Ctrl+C.
#[napi]
pub fn run_broker() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match parse_robot_bus_config(&args).map_err(anyhow_err)? {
        None => {
            print!("{}", robot_bus_broker_help());
            return Ok(());
        }
        Some(config) => config,
    };

    let flag = Arc::new(AtomicBool::new(false));
    shutdown::install(flag.clone());

    println!("robot-bus-broker starting message + service + action buses + gRPC + console…");
    let broker = RustRobotBusBroker::start(config).map_err(anyhow_err)?;
    let mut broker = RobotBusBroker {
        inner: Some(broker),
    };
    println!(
        "gRPC / gRPC-Web listening on http://{}",
        broker.grpc_listen()?
    );
    if let Some(addr) = broker.console_listen()? {
        println!("Web console listening on http://{addr}");
    }

    while !flag.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }

    broker.stop()?;
    println!("robot-bus-broker stopped");
    Ok(())
}

#[napi]
pub struct TfBuffer {
    buffer: SharedBuffer,
}

#[napi]
impl TfBuffer {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(std::sync::Mutex::new(RustTfBuffer::new())),
        }
    }

    #[napi]
    pub fn clear(&self) -> Result<()> {
        self.buffer
            .lock()
            .map_err(|_| Error::from_reason("tf buffer lock poisoned"))?
            .clear();
        Ok(())
    }

    /// Ingest a `tf2_msgs/TFMessage` protobuf. `isStatic` marks `/tf_static` traffic.
    #[napi]
    pub fn set_transform_msg(&self, data: Buffer, is_static: bool) -> Result<()> {
        let msg = TfMessage::decode(data.as_ref())
            .map_err(|e| Error::from_reason(format!("decode TFMessage: {e}")))?;
        self.buffer
            .lock()
            .map_err(|_| Error::from_reason("tf buffer lock poisoned"))?
            .set_transform_msg(&msg, is_static);
        Ok(())
    }

    /// Lookup transform of `source` in `target` as `TransformStamped` protobuf bytes.
    #[napi]
    pub fn lookup_transform(&self, target: String, source: String) -> Result<Buffer> {
        let stamped = self
            .buffer
            .lock()
            .map_err(|_| Error::from_reason("tf buffer lock poisoned"))?
            .lookup_transform(&target, &source, None)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Buffer::from(stamped.encode_to_vec()))
    }

    #[napi]
    pub fn can_transform(&self, target: String, source: String) -> Result<bool> {
        Ok(self
            .buffer
            .lock()
            .map_err(|_| Error::from_reason("tf buffer lock poisoned"))?
            .can_transform(&target, &source))
    }

    #[napi]
    pub fn frames(&self) -> Result<Vec<String>> {
        Ok(self
            .buffer
            .lock()
            .map_err(|_| Error::from_reason("tf buffer lock poisoned"))?
            .frames())
    }
}

#[napi]
pub struct TfListener {
    inner: RustTfListener,
}

#[napi]
impl TfListener {
    #[napi(constructor)]
    pub fn new(node: &mut Node, tf_topic: Option<String>, tf_static_topic: Option<String>) -> Result<Self> {
        let tf = tf_topic.as_deref().unwrap_or("/tf");
        let tf_static = tf_static_topic.as_deref().unwrap_or("/tf_static");
        Ok(Self {
            inner: RustTfListener::new(&mut node.inner, tf, tf_static).map_err(bus_err)?,
        })
    }

    #[napi(factory)]
    pub fn with_defaults(node: &mut Node) -> Result<Self> {
        Ok(Self {
            inner: RustTfListener::with_defaults(&mut node.inner).map_err(bus_err)?,
        })
    }

    /// Shared buffer handle (Arc clone).
    #[napi]
    pub fn buffer(&self) -> TfBuffer {
        TfBuffer {
            buffer: self.inner.buffer(),
        }
    }
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
