//! Low-level pub/sub and topic publisher wrappers.

use std::time::Duration;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use robot_bus::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use robot_bus::runtime::TopicPublisherRaw as RustTopicPublisher;

use crate::util::bus_err;

#[napi]
pub struct Publisher {
    pub(crate) inner: RustPublisher,
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
    pub(crate) inner: RustSubscriber,
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

#[napi]
pub struct TopicPublisher {
    pub(crate) inner: RustTopicPublisher,
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
