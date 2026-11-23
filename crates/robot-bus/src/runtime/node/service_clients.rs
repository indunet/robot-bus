use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use prost::Message;

use crate::errors::{BusError, Result};
use crate::runtime::console_ready::{self, ReadyKind};
use crate::runtime::topology_register::TopologyEndpointGuard;
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::WsClientContext;
use crate::service_bus::ServiceClient as BusServiceClient;
use crate::typed::Service;
use crate::zmq_helpers::HighWaterMark;

/// Service server handle returned by [`Node::create_service`] / [`Node::create_service_raw`].
#[derive(Clone, Debug)]
pub struct NodeService {
    pub(super) id: u64,
    pub(super) service_name: String,
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
    pub(super) inner: ServiceClientInner,
    pub(super) service_name: String,
    pub(super) console_url: Option<String>,
    pub(super) _topology: Option<Arc<TopologyEndpointGuard>>,
}

pub(super) enum ServiceClientInner {
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
    pub(super) inner: NodeServiceClientRaw,
    pub(super) _marker: PhantomData<S>,
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
