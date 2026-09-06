use crate::errors::{BusError, Result};
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::WsRuntime;
use crate::transports::{
    ACTION_BACKEND_CHANNEL, ACTION_FRONTEND_CHANNEL, SERVICE_BACKEND_CHANNEL,
    SERVICE_FRONTEND_CHANNEL, XPUB_CHANNEL, XSUB_CHANNEL, inproc_endpoint_with_prefix,
    ipc_endpoint_in,
};

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
    /// WebSocket RPC gateway base URL when `transport == "ws"` (e.g. `http://127.0.0.1:15560`).
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

    /// WebSocket RPC gateway (native + browser `/ws-rpc`) on the local broker (`http://127.0.0.1:15560`).
    #[cfg(feature = "ws")]
    pub fn ws() -> Self {
        Self::ws_at(WsRuntime::default_url())
    }

    /// WebSocket RPC gateway at `url` (e.g. `http://127.0.0.1:15560`); browsers use `ws(s)://…/ws-rpc`.
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
                 (GET http://127.0.0.1:15560/api/v1/discover) or set endpoints explicitly"
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

pub(super) fn ws_mode_unsupported(op: &str) -> BusError {
    BusError::Protocol(format!(
        "{op} is not supported in WebSocket RPC node mode (client: subscribe / publish / call service / call action; no servers)"
    ))
}
