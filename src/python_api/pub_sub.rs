//! Low-level Publisher / Subscriber and TopicPublisher.

use std::time::Duration;

use pyo3::prelude::*;

use crate::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use crate::runtime::TopicPublisherRaw as RustTopicPublisher;

use super::util::bus_err;

#[pyclass(name = "Publisher", unsendable)]
pub(crate) struct PyPublisher {
    pub(crate) inner: RustPublisher,
}

#[pymethods]
impl PyPublisher {
    #[new]
    #[pyo3(signature = (endpoint=None))]
    fn new(endpoint: Option<&str>) -> PyResult<Self> {
        Ok(Self {
            inner: RustPublisher::new(endpoint).map_err(bus_err)?,
        })
    }

    fn publish(&self, topic: &str, payload: &[u8]) -> PyResult<()> {
        self.inner.publish(topic, payload).map_err(bus_err)
    }

    #[getter]
    fn endpoint(&self) -> &str {
        self.inner.endpoint()
    }
}

#[pyclass(name = "Subscriber", unsendable)]
pub(crate) struct PySubscriber {
    pub(crate) inner: RustSubscriber,
}

#[pymethods]
impl PySubscriber {
    #[new]
    #[pyo3(signature = (endpoint=None))]
    fn new(endpoint: Option<&str>) -> PyResult<Self> {
        Ok(Self {
            inner: RustSubscriber::new(endpoint).map_err(bus_err)?,
        })
    }

    fn subscribe(&self, topic: &str) -> PyResult<()> {
        self.inner.subscribe(topic).map_err(bus_err)
    }

    fn unsubscribe(&self, topic: &str) -> PyResult<()> {
        self.inner.unsubscribe(topic).map_err(bus_err)
    }

    /// Return `(topic, payload)`. `timeout` is seconds; `None` blocks forever.
    #[pyo3(signature = (timeout=None))]
    fn receive(&self, timeout: Option<f64>) -> PyResult<(String, Vec<u8>)> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.receive(timeout).map_err(bus_err)
    }

    #[getter]
    fn endpoint(&self) -> &str {
        self.inner.endpoint()
    }
}

#[pyclass(name = "TopicPublisher", unsendable)]
pub(crate) struct PyTopicPublisher {
    pub(crate) inner: RustTopicPublisher,
}

#[pymethods]
impl PyTopicPublisher {
    #[getter]
    fn topic(&self) -> &str {
        self.inner.topic()
    }

    fn publish(&self, payload: &[u8]) -> PyResult<()> {
        self.inner.publish(payload).map_err(bus_err)
    }
}
