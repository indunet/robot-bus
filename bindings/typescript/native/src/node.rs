//! Context and Node napi bindings.

use std::sync::Arc;
use std::time::Duration;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{
    ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi_derive::napi;
use robot_bus::discovery::{wait as discover_wait, DiscoverOpts as RustDiscoverOpts};
use robot_bus::runtime::{
    ActionGoalHandler, Context as RustContext, Node as RustNode,
    NodeOptions as RustNodeOptions, Parameter, ServiceHandler, TimerCallback,
};

use crate::handles::{
    ActionServerHandle, JsCallbackGroup, JsCallbackGroupType, ServiceHandle, ShutdownHandle,
    SubscriptionHandle, TimerHandle,
};
use crate::message::TopicPublisher;
use crate::rpc::{ActionClient, ActionPhaseReply, ServiceClient};
use crate::util::{
    bus_err, node_options, parameter_to_js, parameter_value_from_js,
};

#[napi(object)]
#[derive(Default)]
pub struct DiscoverNodeOptions {
    pub transport: Option<String>,
    pub api_url: Option<String>,
    pub broker_id: Option<String>,
    pub timeout_secs: Option<f64>,
}


type MsgTsfn = ThreadsafeFunction<(String, Vec<u8>), ErrorStrategy::Fatal>;
type VoidTsfn = ThreadsafeFunction<(), ErrorStrategy::Fatal>;
type ServiceTsfn = ThreadsafeFunction<Vec<u8>, ErrorStrategy::CalleeHandled>;
type ActionTsfn = ThreadsafeFunction<Vec<u8>, ErrorStrategy::CalleeHandled>;

/// Shared ZeroMQ runtime context (required for same-process inproc).
#[napi]
pub struct Context {
    pub(crate) inner: RustContext,
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
    pub(crate) inner: RustNode,
}

#[napi]
impl Node {
    #[napi(constructor)]
    pub fn new(
        name: String,
        host: Option<String>,
        transport: Option<String>,
        ws_url: Option<String>,
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
                    ws_url,
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
                inner: RustNode::inproc_at_with_context(&context.inner, name, p),
            },
            None => Self {
                inner: RustNode::inproc_with_context(&context.inner, name),
            },
        }
    }

    #[napi(factory)]
    pub fn with_context(
        context: &Context,
        name: String,
        host: Option<String>,
        transport: Option<String>,
        ws_url: Option<String>,
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
            inner: RustNode::with_context_options(
                &context.inner,
                name,
                node_options(
                    &host,
                    &transport,
                    ws_url,
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
    pub fn ws(name: String) -> Self {
        Self {
            inner: RustNode::ws(name),
        }
    }

    #[napi(factory)]
    pub fn ws_at(name: String, url: String) -> Self {
        Self {
            inner: RustNode::ws_at(name, url),
        }
    }

    /// Discover a broker via HTTP `GET /api/v1/discover`, then connect with `transport`.
    #[napi(factory)]
    pub fn discover(name: String, options: Option<DiscoverNodeOptions>) -> Result<Self> {
        let o = options.unwrap_or_default();
        let transport = o.transport.unwrap_or_else(|| "tcp".into());
        let base = match transport.as_str() {
            "tcp" => RustNodeOptions::tcp(),
            "ipc" => RustNodeOptions::ipc(),
            "inproc" => RustNodeOptions::inproc(),
            "ws" => RustNodeOptions::ws(),
            other => {
                return Err(Error::from_reason(format!("unknown transport {other:?}")));
            }
        };
        let mut opts = RustDiscoverOpts {
            broker_id: o.broker_id.filter(|s| !s.is_empty()),
            ..Default::default()
        };
        if let Some(url) = o.api_url {
            if !url.is_empty() {
                opts.api_url = url;
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
    pub fn declare_parameter(&mut self, env: Env, name: String, value: Unknown) -> Result<Unknown> {
        let param = self
            .inner
            .declare_parameter(name, parameter_value_from_js(value)?)
            .map_err(bus_err)?;
        parameter_to_js(&env, param)
    }

    #[napi]
    pub fn get_parameter(&self, env: Env, name: String) -> Result<Unknown> {
        let param = self.inner.get_parameter(&name).map_err(bus_err)?;
        parameter_to_js(&env, param)
    }

    #[napi]
    pub fn set_parameter(&mut self, name: String, value: Unknown) -> Result<()> {
        self.inner
            .set_parameter(Parameter::new(name, parameter_value_from_js(value)?))
            .map_err(bus_err)
    }

    #[napi]
    pub fn has_parameter(&self, name: String) -> bool {
        self.inner.has_parameter(&name)
    }

    #[napi]
    pub fn undeclare_parameter(&mut self, name: String) -> Result<()> {
        self.inner.undeclare_parameter(&name).map_err(bus_err)
    }

    /// ROS 2–style list: `{ names: string[], prefixes: string[] }`.
    #[napi]
    pub fn list_parameters(
        &self,
        env: Env,
        prefixes: Option<Vec<String>>,
        depth: Option<u32>,
    ) -> Result<Unknown> {
        let owned = prefixes.unwrap_or_default();
        let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        let result = self
            .inner
            .list_parameters(&refs, u64::from(depth.unwrap_or(0)));
        let mut obj = env.create_object()?;
        let mut names = env.create_array_with_length(result.names.len())?;
        for (i, name) in result.names.iter().enumerate() {
            names.set_element(i as u32, env.create_string(name)?)?;
        }
        let mut prefixes_arr = env.create_array_with_length(result.prefixes.len())?;
        for (i, prefix) in result.prefixes.iter().enumerate() {
            prefixes_arr.set_element(i as u32, env.create_string(prefix)?)?;
        }
        obj.set_named_property("names", names)?;
        obj.set_named_property("prefixes", prefixes_arr)?;
        Ok(obj.into_unknown())
    }

    #[napi]
    pub fn list_all_parameters(&self, env: Env) -> Result<Vec<Unknown>> {
        let mut out = Vec::new();
        for param in self.inner.list_all_parameters() {
            out.push(parameter_to_js(&env, param)?);
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
    pub fn create_publisher(
        &mut self,
        topic: String,
        qos_depth: Option<i32>,
    ) -> Result<TopicPublisher> {
        use robot_bus::runtime::QosProfile;
        let inner = match qos_depth.filter(|d| *d > 0) {
            Some(depth) => self
                .inner
                .create_publisher_raw_with_qos(&topic, QosProfile::keep_last(depth)),
            None => self.inner.create_publisher_raw(&topic),
        }
        .map_err(bus_err)?;
        Ok(TopicPublisher { inner })
    }

    /// Register a subscription callback `(topic: string, payload: Buffer) => void`.
    #[napi]
    pub fn create_subscription(
        &mut self,
        topic: String,
        callback: JsFunction,
        callback_group: Option<&JsCallbackGroup>,
        qos_depth: Option<i32>,
    ) -> Result<SubscriptionHandle> {
        use robot_bus::runtime::QosProfile;
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
        let handle = match qos_depth.filter(|d| *d > 0) {
            Some(depth) => self.inner.create_subscription_raw_with_qos(
                &topic,
                QosProfile::keep_last(depth),
                cb,
                group,
            ),
            None => self.inner.create_subscription_raw(&topic, cb, group),
        }
        .map_err(bus_err)?;
        Ok(SubscriptionHandle {
            inner: Some(handle),
        })
    }

    #[napi]
    pub fn destroy_subscription(&mut self, handle: &mut SubscriptionHandle) -> Result<()> {
        let Some(h) = handle.inner.take() else {
            return Ok(());
        };
        self.inner.destroy_subscription(h).map_err(bus_err)
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

    /// Alias for [`create_timer`](Self::create_timer) (ROS 2 `create_wall_timer`).
    #[napi]
    pub fn create_wall_timer(
        &mut self,
        period: f64,
        callback: JsFunction,
        callback_group: Option<&JsCallbackGroup>,
    ) -> Result<TimerHandle> {
        self.create_timer(period, callback, callback_group)
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
    ) -> Result<ServiceHandle> {
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
        let handle = self
            .inner
            .create_service_raw(&service_name, cb, group)
            .map_err(bus_err)?;
        Ok(ServiceHandle {
            inner: Some(handle),
        })
    }

    #[napi]
    pub fn destroy_service(&mut self, handle: &mut ServiceHandle) -> Result<()> {
        let Some(h) = handle.inner.take() else {
            return Ok(());
        };
        self.inner.destroy_service(&h).map_err(bus_err)
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
    ) -> Result<ActionServerHandle> {
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
        let handle = self
            .inner
            .create_action_server_raw(&action_name, cb, group)
            .map_err(bus_err)?;
        Ok(ActionServerHandle {
            inner: Some(handle),
        })
    }

    #[napi]
    pub fn destroy_action_server(&mut self, handle: &mut ActionServerHandle) -> Result<()> {
        let Some(h) = handle.inner.take() else {
            return Ok(());
        };
        self.inner.destroy_action_server(&h).map_err(bus_err)
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

    /// Wait for one message on `topic`. Returns payload Buffer or `null` on timeout.
    #[napi]
    pub fn wait_for_message(
        &mut self,
        topic: String,
        timeout: Option<f64>,
    ) -> Result<Option<Buffer>> {
        let timeout = timeout.map(Duration::from_secs_f64);
        match self.inner.wait_for_message(&topic, timeout).map_err(bus_err)? {
            Some(payload) => Ok(Some(Buffer::from(payload))),
            None => Ok(None),
        }
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
