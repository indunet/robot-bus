//! PyO3 bindings for the v1 Python SDK surface.

use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::errors::BusError;
use crate::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use crate::runtime::{
    Node as RustNode, ShutdownHandle as RustShutdownHandle, TimerHandle as RustTimerHandle,
};
use crate::transports;

fn bus_err(err: BusError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn map_endpoint_err(err: String) -> PyErr {
    PyRuntimeError::new_err(err)
}

#[pyfunction]
#[pyo3(signature = (host="localhost", transport="tcp"))]
fn message_xsub_endpoint(host: &str, transport: &str) -> PyResult<String> {
    transports::message_xsub_endpoint(host, transport).map_err(map_endpoint_err)
}

#[pyfunction]
#[pyo3(signature = (host="localhost", transport="tcp"))]
fn message_xpub_endpoint(host: &str, transport: &str) -> PyResult<String> {
    transports::message_xpub_endpoint(host, transport).map_err(map_endpoint_err)
}

#[pyclass(name = "Publisher", unsendable)]
struct PyPublisher {
    inner: RustPublisher,
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
struct PySubscriber {
    inner: RustSubscriber,
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

#[pyclass(name = "ShutdownHandle")]
#[derive(Clone)]
struct PyShutdownHandle {
    inner: RustShutdownHandle,
}

#[pymethods]
impl PyShutdownHandle {
    fn shutdown(&self) {
        self.inner.shutdown();
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }
}

#[pyclass(name = "TimerHandle")]
#[derive(Clone, Copy)]
struct PyTimerHandle {
    inner: RustTimerHandle,
}

#[pyclass(name = "Node", unsendable)]
struct PyNode {
    inner: RustNode,
}

#[pymethods]
impl PyNode {
    #[new]
    #[pyo3(signature = (name, namespace=None))]
    fn new(name: String, namespace: Option<String>) -> Self {
        match namespace {
            Some(ns) => Self {
                inner: RustNode::with_namespace(name, ns),
            },
            None => Self {
                inner: RustNode::new(name),
            },
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn namespace(&self) -> &str {
        self.inner.namespace()
    }

    fn fully_qualified_name(&self) -> String {
        self.inner.fully_qualified_name()
    }

    fn resolve_name(&self, name: &str) -> String {
        self.inner.resolve_name(name)
    }

    #[pyo3(signature = (endpoint=None))]
    fn create_publisher(&mut self, endpoint: Option<&str>) -> PyResult<()> {
        self.inner.create_publisher(endpoint).map_err(bus_err)
    }

    fn publish(&self, topic: &str, payload: &[u8]) -> PyResult<()> {
        self.inner.publish(topic, payload).map_err(bus_err)
    }

    /// Register a subscription callback `callback(topic: str, payload: bytes)`.
    #[pyo3(signature = (topic, callback, endpoint=None))]
    fn create_subscription(
        &mut self,
        topic: &str,
        callback: Py<PyAny>,
        endpoint: Option<&str>,
    ) -> PyResult<()> {
        let cb: crate::runtime::MessageCallback = Arc::new(move |topic, payload| {
            Python::with_gil(|py| {
                let payload = PyBytes::new(py, payload);
                if let Err(err) = callback.call1(py, (topic, payload)) {
                    err.print(py);
                }
            });
        });
        self.inner
            .create_subscription(topic, cb, endpoint)
            .map_err(bus_err)
    }

    /// Periodic timer; `callback()` takes no arguments. `period` is seconds.
    fn create_timer(&mut self, period: f64, callback: Py<PyAny>) -> PyResult<PyTimerHandle> {
        let cb: crate::runtime::TimerCallback = Arc::new(move || {
            Python::with_gil(|py| {
                if let Err(err) = callback.call0(py) {
                    err.print(py);
                }
            });
        });
        let handle = self
            .inner
            .create_timer(Duration::from_secs_f64(period), cb)
            .map_err(bus_err)?;
        Ok(PyTimerHandle { inner: handle })
    }

    fn cancel_timer(&mut self, handle: PyTimerHandle) -> PyResult<()> {
        self.inner.cancel_timer(handle.inner).map_err(bus_err)
    }

    fn shutdown_handle(&self) -> PyShutdownHandle {
        PyShutdownHandle {
            inner: self.inner.shutdown_handle(),
        }
    }

    fn shutdown(&self) {
        self.inner.shutdown();
    }

    /// Poll once. `timeout` is seconds; `None` uses the runtime default.
    #[pyo3(signature = (timeout=None))]
    fn spin_once(&mut self, py: Python<'_>, timeout: Option<f64>) -> PyResult<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        py.allow_threads(|| self.inner.spin_once(timeout).map_err(bus_err))
    }

    fn spin(&mut self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| self.inner.spin().map_err(bus_err))
    }
}

#[pymodule]
fn robot_bus(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPublisher>()?;
    m.add_class::<PySubscriber>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyShutdownHandle>()?;
    m.add_class::<PyTimerHandle>()?;
    m.add_function(wrap_pyfunction!(message_xsub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(message_xpub_endpoint, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
