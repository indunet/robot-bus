//! PyO3 bindings for the v1 Python SDK surface.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use pyo3::exceptions::{PyRuntimeError, PyTimeoutError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};

use crate::action_bus::ActionKind;
use crate::broker::{
    RobotBusBroker as RustRobotBusBroker, RobotBusConfig, apply_federation_opts,
    parse_robot_bus_config, robot_bus_broker_help,
};
use crate::discovery::{DiscoverOpts as RustDiscoverOpts, wait as discover_wait};
use crate::errors::BusError;
use crate::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use crate::runtime::{
    ActionGoalHandler, CallbackGroup, CallbackGroupType, Context as RustContext,
    MultiThreadedExecutor as RustMultiThreadedExecutor, Node as RustNode,
    NodeActionClientRaw as RustNodeActionClient, NodeOptions as RustNodeOptions,
    NodeServiceClientRaw as RustNodeServiceClient, ParameterValue, RawActionFeedbackCallback,
    RawGoalHandle as RustRawGoalHandle, ShutdownHandle as RustShutdownHandle,
    SingleThreadedExecutor as RustSingleThreadedExecutor, TimerHandle as RustTimerHandle,
    TopicPublisherRaw as RustTopicPublisher,
};
use crate::shutdown;
use crate::transports;

fn bus_err(err: BusError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn parameter_value_from_py(value: &Bound<'_, PyAny>) -> PyResult<ParameterValue> {
    if let Ok(v) = value.extract::<bool>() {
        return Ok(ParameterValue::Bool(v));
    }
    if let Ok(v) = value.extract::<i64>() {
        return Ok(ParameterValue::Integer(v));
    }
    if let Ok(v) = value.extract::<f64>() {
        return Ok(ParameterValue::Double(v));
    }
    if let Ok(v) = value.extract::<String>() {
        return Ok(ParameterValue::String(v));
    }
    Err(PyRuntimeError::new_err(
        "parameter value must be bool, int, float, or str",
    ))
}

fn parameter_value_to_py(py: Python<'_>, value: ParameterValue) -> PyResult<PyObject> {
    use pyo3::IntoPyObjectExt;
    match value {
        ParameterValue::Bool(v) => v.into_py_any(py),
        ParameterValue::Integer(v) => v.into_py_any(py),
        ParameterValue::Double(v) => v.into_py_any(py),
        ParameterValue::String(v) => v.into_py_any(py),
    }
}

/// Run blocking ZMQ/gRPC I/O without holding the GIL.
///
/// SAFETY: `f` must not touch Python objects; the pointed-to value must remain
/// valid and unused by other threads for the duration of `f`.
unsafe fn allow_threads_io<'py, T, R>(
    py: Python<'py>,
    value: &T,
    f: impl FnOnce(&T) -> R + Send,
) -> R
where
    R: Send,
{
    let ptr = value as *const T as usize;
    py.allow_threads(move || f(unsafe { &*(ptr as *const T) }))
}

fn anyhow_err(err: anyhow::Error) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn map_endpoint_err(err: String) -> PyErr {
    PyRuntimeError::new_err(err)
}

fn action_message_to_py(
    py: Python<'_>,
    msg: crate::action_bus::ActionMessage,
) -> PyResult<PyObject> {
    let dict = pyo3::types::PyDict::new(py);
    dict.set_item(
        "kind",
        match msg.kind {
            ActionKind::Goal => "GOAL",
            ActionKind::Feedback => "FEEDBACK",
            ActionKind::Result => "RESULT",
            ActionKind::Cancel => "CANCEL",
        },
    )?;
    dict.set_item("body", PyBytes::new(py, &msg.body))?;
    dict.set_item("goal_id", &msg.goal_id)?;
    dict.set_item("action_name", &msg.action_name)?;
    Ok(dict.into())
}

fn py_node_options(
    host: &str,
    transport: &str,
    grpc_url: Option<String>,
    message_xsub: Option<String>,
    message_xpub: Option<String>,
    service_frontend: Option<String>,
    service_backend: Option<String>,
    action_backend: Option<String>,
    action_frontend: Option<String>,
) -> PyResult<crate::runtime::NodeOptions> {
    if transport == "grpc" {
        #[cfg(feature = "grpc")]
        {
            return Ok(match grpc_url {
                Some(url) => RustNodeOptions::grpc_at(url),
                None => RustNodeOptions::grpc(),
            });
        }
        #[cfg(not(feature = "grpc"))]
        {
            let _ = grpc_url;
            return Err(PyRuntimeError::new_err(
                "transport=\"grpc\" requires the grpc feature",
            ));
        }
    }
    if grpc_url.is_some() {
        return Err(PyRuntimeError::new_err(
            "grpc_url is only valid when transport=\"grpc\"",
        ));
    }
    Ok(crate::runtime::NodeOptions {
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

fn py_discover_options(
    transport: &str,
    domain_id: u32,
    broker_id: Option<String>,
    multicast_addr: Option<&str>,
    multicast_port: Option<u16>,
    timeout: f64,
) -> PyResult<RustNodeOptions> {
    let base = match transport {
        "tcp" => RustNodeOptions::tcp(),
        "ipc" => RustNodeOptions::ipc(),
        "inproc" => RustNodeOptions::inproc(),
        "grpc" => {
            #[cfg(feature = "grpc")]
            {
                RustNodeOptions::grpc()
            }
            #[cfg(not(feature = "grpc"))]
            {
                return Err(PyRuntimeError::new_err(
                    "transport=\"grpc\" requires the grpc feature",
                ));
            }
        }
        other => {
            return Err(PyRuntimeError::new_err(format!(
                "unknown transport {other:?}"
            )));
        }
    };
    let mut opts = RustDiscoverOpts {
        domain_id,
        broker_id: broker_id.filter(|s| !s.is_empty()),
        ..Default::default()
    };
    if let Some(addr) = multicast_addr {
        if !addr.is_empty() {
            opts.multicast_addr = addr
                .parse()
                .map_err(|e| PyRuntimeError::new_err(format!("invalid multicast_addr: {e}")))?;
        }
    }
    if let Some(port) = multicast_port {
        if port != 0 {
            opts.multicast_port = port;
        }
    }
    if timeout > 0.0 {
        opts.timeout = Duration::from_secs_f64(timeout);
    }
    discover_wait(opts)
        .and_then(|ann| ann.apply(base))
        .map_err(bus_err)
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

#[pyclass(name = "ServiceClient", unsendable)]
struct PyNodeServiceClient {
    inner: RustNodeServiceClient,
}

#[pymethods]
impl PyNodeServiceClient {
    #[getter]
    fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    /// Call the bound service. `timeout` is seconds; `None` waits indefinitely.
    #[pyo3(signature = (body, timeout=None))]
    fn call(&self, py: Python<'_>, body: &[u8], timeout: Option<f64>) -> PyResult<Vec<u8>> {
        let timeout = timeout.map(Duration::from_secs_f64);
        // Release GIL so peer Node Python handlers can run on the spin thread.
        unsafe {
            allow_threads_io(py, &self.inner, |inner| inner.call(body, timeout)).map_err(bus_err)
        }
    }
}

#[pyclass(name = "ActionClient", unsendable)]
struct PyNodeActionClient {
    inner: RustNodeActionClient,
}

#[pyclass(name = "ActionGoalHandle")]
struct PyActionGoalHandle {
    inner: RustRawGoalHandle,
}

#[pymethods]
impl PyActionGoalHandle {
    #[getter]
    fn goal_id(&self) -> &str {
        self.inner.goal_id()
    }

    #[getter]
    fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    /// Wait for the RESULT body. The action timeout starts when the goal is sent.
    ///
    /// If supplied, `timeout` limits only this call; the goal continues running.
    #[pyo3(signature = (timeout=None))]
    fn result(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<Vec<u8>> {
        let msg = match timeout {
            None => unsafe {
                allow_threads_io(py, &self.inner, RustRawGoalHandle::wait_result)
                    .map_err(bus_err)?
            },
            Some(seconds) => {
                let timeout = Duration::from_secs_f64(seconds);
                let handle = self.inner.clone();
                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                thread::spawn(move || {
                    let _ = tx.send(handle.wait_result());
                });
                py.allow_threads(move || rx.recv_timeout(timeout))
                    .map_err(|err| match err {
                        std::sync::mpsc::RecvTimeoutError::Timeout => {
                            PyTimeoutError::new_err("action result timed out")
                        }
                        std::sync::mpsc::RecvTimeoutError::Disconnected => {
                            PyRuntimeError::new_err("action result waiter disconnected")
                        }
                    })?
                    .map_err(bus_err)?
            }
        };
        Ok(msg.body)
    }

    /// Best-effort cancellation. This does not wait for server acknowledgement.
    #[pyo3(signature = (body=None))]
    fn cancel(&self, body: Option<&[u8]>) -> PyResult<()> {
        match body {
            Some(body) => self.inner.cancel_with_body(body),
            None => self.inner.cancel(),
        }
        .map_err(bus_err)
    }
}

#[pymethods]
impl PyNodeActionClient {
    #[getter]
    fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    /// Send a goal and immediately return a live ActionGoalHandle.
    ///
    /// `feedback_callback`, when present, is called as `callback(body: bytes)`.
    #[pyo3(signature = (body, goal_id=None, timeout=None, feedback_callback=None))]
    fn send_goal(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<f64>,
        feedback_callback: Option<Py<PyAny>>,
    ) -> PyResult<PyActionGoalHandle> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let callback = feedback_callback.map(|callback| {
            Arc::new(move |message: &crate::action_bus::ActionMessage| {
                Python::with_gil(|py| {
                    if let Err(err) = callback.call1(py, (PyBytes::new(py, &message.body),)) {
                        err.print(py);
                    }
                });
            }) as RawActionFeedbackCallback
        });
        Ok(PyActionGoalHandle {
            inner: self
                .inner
                .send_goal(body, goal_id, timeout, callback)
                .map_err(bus_err)?,
        })
    }

    /// Compatibility helper that waits for and collects FEEDBACK/RESULT messages.
    #[pyo3(signature = (body, goal_id=None, timeout=None))]
    fn send_goal_and_wait(
        &self,
        py: Python<'_>,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<f64>,
    ) -> PyResult<PyObject> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let messages = unsafe {
            allow_threads_io(py, &self.inner, |inner| {
                inner.send_goal_and_wait(body, goal_id, timeout)
            })
        }
        .map_err(bus_err)?;
        let list = pyo3::types::PyList::empty(py);
        for msg in messages {
            list.append(action_message_to_py(py, msg)?)?;
        }
        Ok(list.into())
    }
}

#[pyclass(name = "Context")]
#[derive(Clone)]
struct PyContext {
    inner: RustContext,
}

#[pymethods]
impl PyContext {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustContext::new(),
        }
    }
}

#[pyclass(name = "SingleThreadedExecutor", unsendable)]
struct PySingleThreadedExecutor {
    inner: RustSingleThreadedExecutor,
}

#[pymethods]
impl PySingleThreadedExecutor {
    #[new]
    #[pyo3(signature = (context=None))]
    fn new(context: Option<&PyContext>) -> Self {
        Self {
            inner: match context {
                Some(c) => RustSingleThreadedExecutor::with_context(c.inner.clone()),
                None => RustSingleThreadedExecutor::new(),
            },
        }
    }

    fn add_node(&self, node: &mut PyNode) -> PyResult<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[pyo3(signature = (
        name,
        host="localhost",
        transport="tcp",
        grpc_url=None,
        message_xsub=None,
        message_xpub=None,
        service_frontend=None,
        service_backend=None,
        action_backend=None,
        action_frontend=None,
    ))]
    fn create_node(
        &self,
        name: String,
        host: &str,
        transport: &str,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> PyResult<PyNode> {
        let options = py_node_options(
            host,
            transport,
            grpc_url,
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        )?;
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

    fn start(&self) -> PyResult<()> {
        self.inner.start().map_err(bus_err)
    }

    fn stop(&self) -> PyResult<()> {
        self.inner.stop().map_err(bus_err)
    }

    fn wait(&self) -> PyResult<()> {
        self.inner.wait().map_err(bus_err)
    }
}

#[pyclass(name = "MultiThreadedExecutor", unsendable)]
struct PyMultiThreadedExecutor {
    inner: RustMultiThreadedExecutor,
}

#[pymethods]
impl PyMultiThreadedExecutor {
    #[new]
    #[pyo3(signature = (num_threads=4, context=None))]
    fn new(num_threads: usize, context: Option<&PyContext>) -> Self {
        Self {
            inner: match context {
                Some(c) => RustMultiThreadedExecutor::with_context(c.inner.clone(), num_threads),
                None => RustMultiThreadedExecutor::new(num_threads),
            },
        }
    }

    fn add_node(&self, node: &mut PyNode) -> PyResult<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[pyo3(signature = (
        name,
        host="localhost",
        transport="tcp",
        grpc_url=None,
        message_xsub=None,
        message_xpub=None,
        service_frontend=None,
        service_backend=None,
        action_backend=None,
        action_frontend=None,
    ))]
    fn create_node(
        &self,
        name: String,
        host: &str,
        transport: &str,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> PyResult<PyNode> {
        let options = py_node_options(
            host,
            transport,
            grpc_url,
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        )?;
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
        grpc_url=None,
        message_xsub=None,
        message_xpub=None,
        service_frontend=None,
        service_backend=None,
        action_backend=None,
        action_frontend=None,
    ))]
    fn new(
        name: String,
        host: &str,
        transport: &str,
        grpc_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: RustNode::with_options(
                name,
                py_node_options(
                    host,
                    transport,
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

    /// TCP to the local broker (`localhost` + default ports).
    #[classmethod]
    #[pyo3(signature = (name, host="localhost"))]
    fn tcp(_cls: &Bound<'_, PyType>, name: String, host: &str) -> Self {
        Self {
            inner: RustNode::with_options(name, RustNodeOptions::tcp_at(host)),
        }
    }

    /// IPC under `/tmp/robot_bus` (or a custom directory).
    #[classmethod]
    #[pyo3(signature = (name, path=None))]
    fn ipc(_cls: &Bound<'_, PyType>, name: String, path: Option<&str>) -> Self {
        let options = match path {
            Some(dir) => RustNodeOptions::ipc_at(dir),
            None => RustNodeOptions::ipc(),
        };
        Self {
            inner: RustNode::with_options(name, options),
        }
    }

    /// Same-process inproc (default prefix `robot_bus`, or a custom prefix).
    ///
    /// Without a shared [`Context`], inproc cannot see an embedded broker in this process.
    #[classmethod]
    #[pyo3(signature = (name, prefix=None))]
    fn inproc(_cls: &Bound<'_, PyType>, name: String, prefix: Option<&str>) -> Self {
        let options = match prefix {
            Some(p) => RustNodeOptions::inproc_at(p),
            None => RustNodeOptions::inproc(),
        };
        Self {
            inner: RustNode::with_options(name, options),
        }
    }

    /// Same-process inproc sharing `context` with an embedded broker.
    #[classmethod]
    #[pyo3(signature = (context, name, prefix=None))]
    fn inproc_with_context(
        _cls: &Bound<'_, PyType>,
        context: &PyContext,
        name: String,
        prefix: Option<&str>,
    ) -> Self {
        Self {
            inner: match prefix {
                Some(p) => RustNode::inproc_at_with_context(context.inner.clone(), name, p),
                None => RustNode::inproc_with_context(context.inner.clone(), name),
            },
        }
    }

    /// gRPC client node talking to the local broker gateway (`http://127.0.0.1:15770`).
    #[cfg(feature = "grpc")]
    #[classmethod]
    #[pyo3(signature = (name,))]
    fn grpc(_cls: &Bound<'_, PyType>, name: String) -> Self {
        Self {
            inner: RustNode::grpc(name),
        }
    }

    /// Discover a broker via UDP multicast, then connect with `transport`.
    ///
    /// Transport is still chosen by the caller (`tcp` / `ipc` / `inproc` / `grpc`);
    /// discovery only fills host / paths / gRPC URL.
    #[classmethod]
    #[pyo3(signature = (
        name,
        transport="tcp",
        *,
        domain_id=0,
        broker_id=None,
        multicast_addr=None,
        multicast_port=None,
        timeout=3.0,
    ))]
    fn discover(
        _cls: &Bound<'_, PyType>,
        name: String,
        transport: &str,
        domain_id: u32,
        broker_id: Option<String>,
        multicast_addr: Option<&str>,
        multicast_port: Option<u16>,
        timeout: f64,
    ) -> PyResult<Self> {
        let options = py_discover_options(
            transport,
            domain_id,
            broker_id,
            multicast_addr,
            multicast_port,
            timeout,
        )?;
        Ok(Self {
            inner: RustNode::with_options(name, options),
        })
    }

    /// gRPC client node talking to `url` (e.g. `http://127.0.0.1:15770`).
    #[cfg(feature = "grpc")]
    #[classmethod]
    #[pyo3(signature = (name, url))]
    fn grpc_at(_cls: &Bound<'_, PyType>, name: String, url: &str) -> Self {
        Self {
            inner: RustNode::grpc_at(name, url),
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    /// Declare a local parameter (`bool` / `int` / `float` / `str`).
    fn declare_parameter(&mut self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner
            .declare_parameter(name, parameter_value_from_py(value)?)
            .map_err(bus_err)
    }

    /// Read a previously declared parameter (returns `bool` / `int` / `float` / `str`).
    fn get_parameter(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        let value = self.inner.get_parameter(name).map_err(bus_err)?;
        parameter_value_to_py(py, value)
    }

    /// Update a declared parameter (type must match).
    fn set_parameter(&mut self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner
            .set_parameter(name, parameter_value_from_py(value)?)
            .map_err(bus_err)
    }

    fn has_parameter(&self, name: &str) -> bool {
        self.inner.has_parameter(name)
    }

    /// All declared parameters as `list[dict]` with keys `name`, `value`.
    fn list_parameters(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = pyo3::types::PyList::empty(py);
        for param in self.inner.list_parameters() {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", &param.name)?;
            dict.set_item("value", parameter_value_to_py(py, param.value)?)?;
            list.append(dict)?;
        }
        Ok(list.into())
    }

    /// Load parameters from a YAML string (declare missing, set existing).
    fn load_parameters_from_yaml_str(&mut self, yaml: &str) -> PyResult<()> {
        self.inner
            .load_parameters_from_yaml_str(yaml)
            .map_err(bus_err)
    }

    /// Load parameters from a YAML file path.
    fn load_parameters_from_yaml_file(&mut self, path: &str) -> PyResult<()> {
        self.inner
            .load_parameters_from_yaml_file(path)
            .map_err(bus_err)
    }

    fn create_callback_group(&self, kind: PyCallbackGroupType) -> PyCallbackGroup {
        PyCallbackGroup {
            inner: self.inner.create_callback_group(kind.into()),
        }
    }

    fn create_publisher(&mut self, topic: &str) -> PyResult<PyTopicPublisher> {
        Ok(PyTopicPublisher {
            inner: self.inner.create_publisher_raw(topic).map_err(bus_err)?,
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
        let group = callback_group.map(|g| &g.inner);
        self.inner
            .create_subscription_raw(topic, cb, group)
            .map_err(bus_err)
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
        let group = callback_group.map(|g| &g.inner);
        let handle = self
            .inner
            .create_timer(Duration::from_secs_f64(period), cb, group)
            .map_err(bus_err)?;
        Ok(PyTimerHandle { inner: handle })
    }

    fn cancel_timer(&mut self, handle: PyTimerHandle) -> PyResult<()> {
        self.inner.cancel_timer(handle.inner).map_err(bus_err)
    }

    /// Register a service server.
    ///
    /// `handler(body: bytes) -> bytes`
    #[pyo3(signature = (service_name, handler, callback_group=None))]
    fn create_service(
        &mut self,
        service_name: &str,
        handler: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
    ) -> PyResult<()> {
        let cb: crate::runtime::ServiceHandler = Arc::new(move |body| {
            Python::with_gil(|py| {
                let args = (PyBytes::new(py, body),);
                match handler.call1(py, args) {
                    Ok(obj) => match obj.extract::<Vec<u8>>(py) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            err.print(py);
                            Vec::new()
                        }
                    },
                    Err(err) => {
                        err.print(py);
                        Vec::new()
                    }
                }
            })
        });
        let group = callback_group.map(|g| &g.inner);
        self.inner
            .create_service_raw(service_name, cb, group)
            .map(|_| ())
            .map_err(bus_err)
    }

    /// Create a service client bound to `service_name` (ROS 2 `create_client`).
    fn create_client(&mut self, service_name: &str) -> PyResult<PyNodeServiceClient> {
        Ok(PyNodeServiceClient {
            inner: self
                .inner
                .create_client_raw(service_name)
                .map_err(bus_err)?,
        })
    }

    /// Register an action server (ROS 2–style `create_action_server`).
    ///
    /// `handler(payload: bytes) -> list[tuple[str, bytes]]`
    /// where each tuple is `(phase, body)` and `phase` is typically `"FEEDBACK"` / `"RESULT"`.
    #[pyo3(signature = (action_name, handler, callback_group=None))]
    fn create_action_server(
        &mut self,
        action_name: &str,
        handler: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
    ) -> PyResult<()> {
        let cb: ActionGoalHandler = Arc::new(move |payload| {
            Python::with_gil(|py| {
                let args = (PyBytes::new(py, payload),);
                match handler.call1(py, args) {
                    Ok(obj) => match obj.extract::<Vec<(String, Vec<u8>)>>(py) {
                        Ok(replies) => replies,
                        Err(err) => {
                            err.print(py);
                            Vec::new()
                        }
                    },
                    Err(err) => {
                        err.print(py);
                        Vec::new()
                    }
                }
            })
        });
        let group = callback_group.map(|g| &g.inner);
        self.inner
            .create_action_server_raw(action_name, cb, group)
            .map(|_| ())
            .map_err(bus_err)
    }

    /// Create an action client bound to `action_name` (ROS 2 `create_action_client`).
    fn create_action_client(&mut self, action_name: &str) -> PyResult<PyNodeActionClient> {
        Ok(PyNodeActionClient {
            inner: self
                .inner
                .create_action_client_raw(action_name)
                .map_err(bus_err)?,
        })
    }

    fn connect_action_client(&mut self) -> PyResult<()> {
        self.inner.connect_action_client().map_err(bus_err)
    }

    fn shutdown_handle(&mut self) -> PyResult<PyShutdownHandle> {
        Ok(PyShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    fn shutdown(&mut self) -> PyResult<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    /// Poll once. `timeout` is seconds; `None` uses the executor default.
    #[pyo3(signature = (timeout=None))]
    fn spin_once(&mut self, timeout: Option<f64>) -> PyResult<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    fn spin(&mut self) -> PyResult<()> {
        self.inner.spin().map_err(bus_err)
    }

    /// Drive the executor on a background thread (ZMQ nodes).
    fn start(&mut self) -> PyResult<()> {
        self.inner.start().map_err(bus_err)
    }

    fn stop(&mut self) -> PyResult<()> {
        self.inner.stop().map_err(bus_err)
    }

    fn wait(&mut self) -> PyResult<()> {
        self.inner.wait().map_err(bus_err)
    }
}

/// In-process broker: message + service + action buses + gRPC on background threads.
#[pyclass(name = "RobotBusBroker", unsendable)]
struct PyRobotBusBroker {
    inner: Option<RustRobotBusBroker>,
}

#[pymethods]
impl PyRobotBusBroker {
    /// Start all buses (and gRPC). Keyword args override [`RobotBusConfig`] defaults.
    #[staticmethod]
    #[pyo3(signature = (
        *,
        context = None,
        message_xsub_bind = None,
        message_xpub_bind = None,
        message_snd_hwm = None,
        message_rcv_hwm = None,
        service_frontend_bind = None,
        service_backend_bind = None,
        service_snd_hwm = None,
        service_rcv_hwm = None,
        service_heartbeat_interval_ms = None,
        service_heartbeat_timeout_ms = None,
        service_pending_timeout_ms = None,
        service_max_pending = None,
        action_frontend_bind = None,
        action_backend_bind = None,
        action_snd_hwm = None,
        action_rcv_hwm = None,
        action_heartbeat_interval_ms = None,
        action_heartbeat_timeout_ms = None,
        action_pending_timeout_ms = None,
        snd_hwm = None,
        rcv_hwm = None,
        heartbeat_interval_ms = None,
        heartbeat_timeout_ms = None,
        tcp_only = false,
        grpc_listen = None,
        cors_origins = None,
        console_listen = None,
        no_console = false,
        broker_id = None,
        message_peers = None,
        service_peers = None,
        action_peers = None,
        domain_id = 0,
        no_discovery = false,
        advertise_host = None,
        discovery_addr = None,
        discovery_port = None,
    ))]
    fn start(
        context: Option<&PyContext>,
        message_xsub_bind: Option<String>,
        message_xpub_bind: Option<String>,
        message_snd_hwm: Option<i32>,
        message_rcv_hwm: Option<i32>,
        service_frontend_bind: Option<String>,
        service_backend_bind: Option<String>,
        service_snd_hwm: Option<i32>,
        service_rcv_hwm: Option<i32>,
        service_heartbeat_interval_ms: Option<u64>,
        service_heartbeat_timeout_ms: Option<u64>,
        service_pending_timeout_ms: Option<u64>,
        service_max_pending: Option<u64>,
        action_frontend_bind: Option<String>,
        action_backend_bind: Option<String>,
        action_snd_hwm: Option<i32>,
        action_rcv_hwm: Option<i32>,
        action_heartbeat_interval_ms: Option<u64>,
        action_heartbeat_timeout_ms: Option<u64>,
        action_pending_timeout_ms: Option<u64>,
        snd_hwm: Option<i32>,
        rcv_hwm: Option<i32>,
        heartbeat_interval_ms: Option<u64>,
        heartbeat_timeout_ms: Option<u64>,
        tcp_only: bool,
        grpc_listen: Option<String>,
        cors_origins: Option<Vec<String>>,
        console_listen: Option<String>,
        no_console: bool,
        broker_id: Option<String>,
        message_peers: Option<Vec<String>>,
        service_peers: Option<Vec<String>>,
        action_peers: Option<Vec<String>>,
        domain_id: u32,
        no_discovery: bool,
        advertise_host: Option<String>,
        discovery_addr: Option<String>,
        discovery_port: Option<u16>,
    ) -> PyResult<Self> {
        let mut config = RobotBusConfig::default();

        if let Some(v) = message_xsub_bind {
            config.message.xsub_bind = normalize_bind(&v);
        }
        if let Some(v) = message_xpub_bind {
            config.message.xpub_bind = normalize_bind(&v);
        }
        if let Some(v) = message_snd_hwm {
            config.message.snd_hwm = v;
        }
        if let Some(v) = message_rcv_hwm {
            config.message.rcv_hwm = v;
        }

        if let Some(v) = service_frontend_bind {
            config.service.frontend_bind = normalize_bind(&v);
        }
        if let Some(v) = service_backend_bind {
            config.service.backend_bind = normalize_bind(&v);
        }
        if let Some(v) = service_snd_hwm {
            config.service.snd_hwm = v;
        }
        if let Some(v) = service_rcv_hwm {
            config.service.rcv_hwm = v;
        }
        if let Some(v) = service_heartbeat_interval_ms {
            config.service.heartbeat_interval_ms = v;
        }
        if let Some(v) = service_heartbeat_timeout_ms {
            config.service.heartbeat_timeout_ms = v;
        }
        if let Some(v) = service_pending_timeout_ms {
            config.service.pending_timeout_ms = v;
        }
        if let Some(v) = service_max_pending {
            config.service.max_pending = v as usize;
        }

        if let Some(v) = action_frontend_bind {
            config.action.frontend_bind = normalize_bind(&v);
        }
        if let Some(v) = action_backend_bind {
            config.action.backend_bind = normalize_bind(&v);
        }
        if let Some(v) = action_snd_hwm {
            config.action.snd_hwm = v;
        }
        if let Some(v) = action_rcv_hwm {
            config.action.rcv_hwm = v;
        }
        if let Some(v) = action_heartbeat_interval_ms {
            config.action.heartbeat_interval_ms = v;
        }
        if let Some(v) = action_heartbeat_timeout_ms {
            config.action.heartbeat_timeout_ms = v;
        }
        if let Some(v) = action_pending_timeout_ms {
            config.action.pending_timeout_ms = v;
        }

        if let Some(v) = snd_hwm {
            config.message.snd_hwm = v;
            config.service.snd_hwm = v;
            config.action.snd_hwm = v;
        }
        if let Some(v) = rcv_hwm {
            config.message.rcv_hwm = v;
            config.service.rcv_hwm = v;
            config.action.rcv_hwm = v;
        }
        if let Some(v) = heartbeat_interval_ms {
            config.service.heartbeat_interval_ms = v;
            config.action.heartbeat_interval_ms = v;
        }
        if let Some(v) = heartbeat_timeout_ms {
            config.service.heartbeat_timeout_ms = v;
            config.action.heartbeat_timeout_ms = v;
        }
        if tcp_only {
            config.message.bind_all_transports = false;
            config.service.bind_all_transports = false;
            config.action.bind_all_transports = false;
        }

        #[cfg(feature = "grpc")]
        {
            if let Some(v) = grpc_listen {
                config.grpc.listen = v
                    .parse()
                    .map_err(|e| PyRuntimeError::new_err(format!("invalid grpc_listen: {e}")))?;
            }
            if let Some(v) = cors_origins {
                config.grpc.cors_origins = v;
            }
        }
        #[cfg(not(feature = "grpc"))]
        {
            let _ = (grpc_listen, cors_origins);
        }

        #[cfg(feature = "console")]
        {
            if no_console {
                config.console.enabled = false;
            }
            if let Some(v) = console_listen {
                config.console.listen = v
                    .parse()
                    .map_err(|e| PyRuntimeError::new_err(format!("invalid console_listen: {e}")))?;
                config.console.enabled = true;
            }
        }
        #[cfg(not(feature = "console"))]
        {
            let _ = (console_listen, no_console);
        }

        apply_federation_opts(
            &mut config,
            broker_id.as_deref(),
            message_peers.as_deref().unwrap_or(&[]),
            service_peers.as_deref().unwrap_or(&[]),
            action_peers.as_deref().unwrap_or(&[]),
        )
        .map_err(anyhow_err)?;

        if no_discovery {
            config.discovery.enabled = false;
        }
        config.discovery.domain_id = domain_id;
        if let Some(v) = advertise_host {
            if !v.is_empty() {
                config.discovery.advertise_host = Some(v);
            }
        }
        if let Some(v) = discovery_addr {
            if !v.is_empty() {
                config.discovery.multicast_addr = v
                    .parse()
                    .map_err(|e| PyRuntimeError::new_err(format!("invalid discovery_addr: {e}")))?;
            }
        }
        if let Some(port) = discovery_port {
            if port != 0 {
                config.discovery.multicast_port = port;
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

    /// Stop all buses (and gRPC) and join their threads. Safe to call more than once.
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

    #[cfg(feature = "grpc")]
    #[getter]
    fn grpc_listen(&self) -> PyResult<String> {
        self.with_broker(|b| b.grpc_listen().to_string())
    }

    #[cfg(feature = "console")]
    #[getter]
    fn console_listen(&self) -> PyResult<Option<String>> {
        self.with_broker(|b| b.console_listen().map(|a| a.to_string()))
    }
}

fn normalize_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
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
    print!("{}", robot_bus_broker_help());
}

/// Blocking CLI entry: start broker and wait for Ctrl+C (or Unix SIGTERM).
///
/// Used by the `robot-bus-broker` console script after `pip install robot-bus`.
#[pyfunction]
fn run_broker(py: Python<'_>) -> PyResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match parse_robot_bus_config(&args).map_err(anyhow_err)? {
        None => {
            print_broker_help();
            return Ok(());
        }
        Some(config) => config,
    };

    let flag = Arc::new(AtomicBool::new(false));
    shutdown::install(flag.clone());

    println!("robot-bus-broker starting message + service + action buses + gRPC + console…");
    let broker = RustRobotBusBroker::start(config).map_err(anyhow_err)?;
    let mut broker = PyRobotBusBroker {
        inner: Some(broker),
    };
    #[cfg(feature = "grpc")]
    println!(
        "gRPC / gRPC-Web listening on http://{}",
        broker.grpc_listen()?
    );
    #[cfg(feature = "console")]
    if let Some(addr) = broker.console_listen()? {
        println!("Web console listening on http://{addr}");
    }

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
    m.add_class::<PyContext>()?;
    m.add_class::<PyCallbackGroupType>()?;
    m.add_class::<PyCallbackGroup>()?;
    m.add_class::<PySingleThreadedExecutor>()?;
    m.add_class::<PyMultiThreadedExecutor>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyTopicPublisher>()?;
    m.add_class::<PyNodeServiceClient>()?;
    m.add_class::<PyNodeActionClient>()?;
    m.add_class::<PyActionGoalHandle>()?;
    m.add_class::<PyShutdownHandle>()?;
    m.add_class::<PyTimerHandle>()?;
    m.add_class::<PyRobotBusBroker>()?;
    m.add_function(wrap_pyfunction!(message_xsub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(message_xpub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(run_broker, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
