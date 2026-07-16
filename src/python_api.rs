//! PyO3 bindings for the v1 Python SDK surface.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::broker::{RobotBusBroker as RustRobotBusBroker, RobotBusConfig};
use crate::errors::BusError;
use crate::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use crate::runtime::{
    CallbackGroup, CallbackGroupType, MultiThreadedExecutor as RustMultiThreadedExecutor,
    Node as RustNode, ShutdownHandle as RustShutdownHandle,
    SingleThreadedExecutor as RustSingleThreadedExecutor, TimerHandle as RustTimerHandle,
    TopicPublisher as RustTopicPublisher,
};
use crate::shutdown;
use crate::transports;

fn bus_err(err: BusError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn anyhow_err(err: anyhow::Error) -> PyErr {
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

#[pyclass(name = "CallbackGroupType", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyCallbackGroupType {
    MutuallyExclusive = 0,
    Reentrant = 1,
}

impl From<PyCallbackGroupType> for CallbackGroupType {
    fn from(value: PyCallbackGroupType) -> Self {
        match value {
            PyCallbackGroupType::MutuallyExclusive => CallbackGroupType::MutuallyExclusive,
            PyCallbackGroupType::Reentrant => CallbackGroupType::Reentrant,
        }
    }
}

#[pyclass(name = "CallbackGroup", unsendable)]
#[derive(Clone)]
struct PyCallbackGroup {
    inner: CallbackGroup,
}

#[pymethods]
impl PyCallbackGroup {
    #[getter]
    fn id(&self) -> u64 {
        self.inner.id()
    }

    #[getter]
    fn kind(&self) -> PyCallbackGroupType {
        match self.inner.kind() {
            CallbackGroupType::MutuallyExclusive => PyCallbackGroupType::MutuallyExclusive,
            CallbackGroupType::Reentrant => PyCallbackGroupType::Reentrant,
        }
    }
}

#[pyclass(name = "TopicPublisher", unsendable)]
struct PyTopicPublisher {
    inner: RustTopicPublisher,
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

#[pyclass(name = "SingleThreadedExecutor", unsendable)]
struct PySingleThreadedExecutor {
    inner: RustSingleThreadedExecutor,
}

#[pymethods]
impl PySingleThreadedExecutor {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustSingleThreadedExecutor::new(),
        }
    }

    fn add_node(&self, node: &mut PyNode) -> PyResult<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[pyo3(signature = (
        name,
        host="localhost",
        transport="tcp",
        message_xsub=None,
        message_xpub=None,
    ))]
    fn create_node(
        &self,
        name: String,
        host: &str,
        transport: &str,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
    ) -> PyResult<PyNode> {
        let options = crate::runtime::NodeOptions {
            host: host.into(),
            transport: transport.into(),
            message_xsub,
            message_xpub,
            ..crate::runtime::NodeOptions::default()
        };
        Ok(PyNode {
            inner: self
                .inner
                .create_node_with_options(name, options)
                .map_err(bus_err)?,
        })
    }

    fn shutdown_handle(&self) -> PyResult<PyShutdownHandle> {
        Ok(PyShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    fn shutdown(&self) -> PyResult<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[pyo3(signature = (timeout=None))]
    fn spin_once(&self, timeout: Option<f64>) -> PyResult<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    fn spin(&self) -> PyResult<()> {
        self.inner.spin().map_err(bus_err)
    }
}

#[pyclass(name = "MultiThreadedExecutor", unsendable)]
struct PyMultiThreadedExecutor {
    inner: RustMultiThreadedExecutor,
}

#[pymethods]
impl PyMultiThreadedExecutor {
    #[new]
    #[pyo3(signature = (num_threads=4))]
    fn new(num_threads: usize) -> Self {
        Self {
            inner: RustMultiThreadedExecutor::new(num_threads),
        }
    }

    fn add_node(&self, node: &mut PyNode) -> PyResult<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[pyo3(signature = (
        name,
        host="localhost",
        transport="tcp",
        message_xsub=None,
        message_xpub=None,
    ))]
    fn create_node(
        &self,
        name: String,
        host: &str,
        transport: &str,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
    ) -> PyResult<PyNode> {
        let options = crate::runtime::NodeOptions {
            host: host.into(),
            transport: transport.into(),
            message_xsub,
            message_xpub,
            ..crate::runtime::NodeOptions::default()
        };
        Ok(PyNode {
            inner: self
                .inner
                .create_node_with_options(name, options)
                .map_err(bus_err)?,
        })
    }

    fn shutdown_handle(&self) -> PyResult<PyShutdownHandle> {
        Ok(PyShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    fn shutdown(&self) -> PyResult<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[pyo3(signature = (timeout=None))]
    fn spin_once(&self, timeout: Option<f64>) -> PyResult<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    fn spin(&self) -> PyResult<()> {
        self.inner.spin().map_err(bus_err)
    }
}

#[pyclass(name = "Node", unsendable)]
struct PyNode {
    inner: RustNode,
}

#[pymethods]
impl PyNode {
    #[new]
    #[pyo3(signature = (
        name,
        host="localhost",
        transport="tcp",
        message_xsub=None,
        message_xpub=None,
    ))]
    fn new(
        name: String,
        host: &str,
        transport: &str,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
    ) -> Self {
        let options = crate::runtime::NodeOptions {
            host: host.into(),
            transport: transport.into(),
            message_xsub,
            message_xpub,
            ..crate::runtime::NodeOptions::default()
        };
        Self {
            inner: RustNode::with_options(name, options),
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn create_callback_group(&self, kind: PyCallbackGroupType) -> PyCallbackGroup {
        PyCallbackGroup {
            inner: self.inner.create_callback_group(kind.into()),
        }
    }

    fn create_publisher(&mut self, topic: &str) -> PyResult<PyTopicPublisher> {
        Ok(PyTopicPublisher {
            inner: self.inner.create_publisher(topic).map_err(bus_err)?,
        })
    }

    /// Register a subscription callback `callback(topic: str, payload: bytes)`.
    #[pyo3(signature = (topic, callback, callback_group=None))]
    fn create_subscription(
        &mut self,
        topic: &str,
        callback: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
    ) -> PyResult<()> {
        let cb: crate::runtime::MessageCallback = Arc::new(move |topic, payload| {
            Python::with_gil(|py| {
                let payload = PyBytes::new(py, payload);
                if let Err(err) = callback.call1(py, (topic, payload)) {
                    err.print(py);
                }
            });
        });
        match callback_group {
            Some(group) => self
                .inner
                .create_subscription_with_group(topic, cb, &group.inner)
                .map_err(bus_err),
            None => self.inner.create_subscription(topic, cb).map_err(bus_err),
        }
    }

    /// Periodic timer; `callback()` takes no arguments. `period` is seconds.
    #[pyo3(signature = (period, callback, callback_group=None))]
    fn create_timer(
        &mut self,
        period: f64,
        callback: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
    ) -> PyResult<PyTimerHandle> {
        let cb: crate::runtime::TimerCallback = Arc::new(move || {
            Python::with_gil(|py| {
                if let Err(err) = callback.call0(py) {
                    err.print(py);
                }
            });
        });
        let handle = match callback_group {
            Some(group) => self
                .inner
                .create_timer_with_group(Duration::from_secs_f64(period), cb, &group.inner)
                .map_err(bus_err)?,
            None => self
                .inner
                .create_timer(Duration::from_secs_f64(period), cb)
                .map_err(bus_err)?,
        };
        Ok(PyTimerHandle { inner: handle })
    }

    fn cancel_timer(&mut self, handle: PyTimerHandle) -> PyResult<()> {
        self.inner.cancel_timer(handle.inner).map_err(bus_err)
    }

    fn shutdown_handle(&self) -> PyResult<PyShutdownHandle> {
        Ok(PyShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    fn shutdown(&self) -> PyResult<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    /// Poll once. `timeout` is seconds; `None` uses the executor default.
    #[pyo3(signature = (timeout=None))]
    fn spin_once(&self, timeout: Option<f64>) -> PyResult<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    fn spin(&self) -> PyResult<()> {
        self.inner.spin().map_err(bus_err)
    }
}

/// In-process broker: message + service + action buses on background threads.
#[pyclass(name = "RobotBusBroker", unsendable)]
struct PyRobotBusBroker {
    inner: Option<RustRobotBusBroker>,
}

#[pymethods]
impl PyRobotBusBroker {
    /// Start all three buses with default binds (same as `robot_bus_broker`).
    #[staticmethod]
    fn start() -> PyResult<Self> {
        let broker = RustRobotBusBroker::start(RobotBusConfig::default()).map_err(anyhow_err)?;
        Ok(Self {
            inner: Some(broker),
        })
    }

    /// Stop all buses and join their threads. Safe to call more than once.
    fn stop(&mut self) -> PyResult<()> {
        if let Some(broker) = self.inner.take() {
            broker.stop().map_err(anyhow_err)?;
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.stop()?;
        Ok(false)
    }

    #[getter]
    fn message_xsub_bind(&self) -> PyResult<String> {
        self.with_broker(|b| b.message.xsub_bind.clone())
    }

    #[getter]
    fn message_xpub_bind(&self) -> PyResult<String> {
        self.with_broker(|b| b.message.xpub_bind.clone())
    }

    #[getter]
    fn service_frontend_bind(&self) -> PyResult<String> {
        self.with_broker(|b| b.service.frontend_bind.clone())
    }

    #[getter]
    fn service_backend_bind(&self) -> PyResult<String> {
        self.with_broker(|b| b.service.backend_bind.clone())
    }

    #[getter]
    fn action_frontend_bind(&self) -> PyResult<String> {
        self.with_broker(|b| b.action.frontend_bind.clone())
    }

    #[getter]
    fn action_backend_bind(&self) -> PyResult<String> {
        self.with_broker(|b| b.action.backend_bind.clone())
    }
}

impl PyRobotBusBroker {
    fn with_broker<T>(&self, f: impl FnOnce(&RustRobotBusBroker) -> T) -> PyResult<T> {
        self.inner
            .as_ref()
            .map(f)
            .ok_or_else(|| PyRuntimeError::new_err("broker already stopped"))
    }
}

fn print_broker_help() {
    println!(
        "robot-bus-broker — start all ZeroMQ buses in one process\n\n\
Usage:\n  robot-bus-broker\n\n\
Starts with default ports and tcp + inproc + ipc on each socket:\n  \
message_bus  15560 / 15561 (XSUB/XPUB proxy)\n  \
service_bus  15662 / 15663 (REQ service broker)\n  \
action_bus   15664 / 15665 (DEALER action broker)\n\n\
Press Ctrl+C to stop all buses.\n\n\
In Python: robot_bus.RobotBusBroker.start() / robot_bus.run_broker()\n"
    );
}

/// Blocking CLI entry: start broker and wait for Ctrl+C (or Unix SIGTERM).
///
/// Used by the `robot-bus-broker` console script after `pip install robot-bus`.
#[pyfunction]
fn run_broker(py: Python<'_>) -> PyResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_broker_help();
        return Ok(());
    }
    if !args.is_empty() {
        return Err(PyRuntimeError::new_err(format!(
            "unknown arguments: {args:?} (try --help)"
        )));
    }

    let flag = Arc::new(AtomicBool::new(false));
    shutdown::install(flag.clone());

    println!("robot-bus-broker starting message + service + action buses…");
    let mut broker = PyRobotBusBroker::start()?;

    loop {
        if flag.load(Ordering::Acquire) {
            break;
        }
        // Raises KeyboardInterrupt on Ctrl+C (works on Windows too).
        if let Err(err) = py.check_signals() {
            if err.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py) {
                break;
            }
            broker.stop()?;
            return Err(err);
        }
        py.allow_threads(|| thread::sleep(Duration::from_millis(50)));
    }

    broker.stop()?;
    println!("robot-bus-broker stopped");
    Ok(())
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPublisher>()?;
    m.add_class::<PySubscriber>()?;
    m.add_class::<PyCallbackGroupType>()?;
    m.add_class::<PyCallbackGroup>()?;
    m.add_class::<PySingleThreadedExecutor>()?;
    m.add_class::<PyMultiThreadedExecutor>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyTopicPublisher>()?;
    m.add_class::<PyShutdownHandle>()?;
    m.add_class::<PyTimerHandle>()?;
    m.add_class::<PyRobotBusBroker>()?;
    m.add_function(wrap_pyfunction!(message_xsub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(message_xpub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(run_broker, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
