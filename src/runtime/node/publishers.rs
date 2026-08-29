use std::marker::PhantomData;
use std::sync::Arc;

use prost::Message;

use crate::errors::{BusError, Result};
use crate::message_bus::Publisher as BusPublisher;
use crate::runtime::topology_register::TopologyEndpointGuard;
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::WsClientContext;
use crate::zmq_helpers::HighWaterMark;

/// Raw (opaque bytes) publisher from [`Node::create_publisher_raw`].
///
/// ZMQ mode shares one underlying bus PUB socket per node; WebSocket RPC mode issues
/// unary `MessageGateway.Publish` RPCs. Each handle remembers its topic.
///
/// Topology registration is shared across clones and unregistered when the last
/// handle drops.
#[derive(Clone)]
pub struct TopicPublisherRaw {
    pub(super) backend: TopicPublisherBackend,
    pub(super) topic: String,
    /// Best-effort console topology registration (kept alive while handles exist).
    pub(super) _topology: Option<Arc<TopologyEndpointGuard>>,
}

#[derive(Clone)]
pub(super) enum TopicPublisherBackend {
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
    pub(super) inner: TopicPublisherRaw,
    pub(super) _marker: PhantomData<M>,
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
