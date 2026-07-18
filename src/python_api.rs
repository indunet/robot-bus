//! PyO3 bindings for the v1 Python SDK surface.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::broker::{
    parse_robot_bus_config, robot_bus_broker_help, RobotBusBroker as RustRobotBusBroker,
    RobotBusConfig,
};
use crate::errors::BusError;
use crate::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use crate::runtime::{
    ActionGoalHandler, CallbackGroup, CallbackGroupType,
    MultiThreadedExecutor as RustMultiThreadedExecutor, Node as RustNode,
    NodeActionClientRaw as RustNodeActionClient, NodeServiceClientRaw as RustNodeServiceClient,
    ShutdownHandle as RustShutdownHandle, SingleThreadedExecutor as RustSingleThreadedExecutor,
    TimerHandle as RustTimerHandle, TopicPublisherRaw as RustTopicPublisher,
};
use crate::action_bus::ActionKind;
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

fn py_node_options(
    host: &str,
    transport: &str,
    message_xsub: Option<String>,
    message_xpub: Option<String>,
    service_frontend: Option<String>,
    service_backend: Option<String>,
    action_backend: Option<String>,
    action_frontend: Option<String>,
) -> crate::runtime::NodeOptions {
    crate::runtime::NodeOptions {
        host: host.into(),
        transport: transport.into(),
        message_xsub,
        message_xpub,
        service_frontend,
        service_backend,
        action_backend,
        action_frontend,
    }
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
    fn call(&self, body: &[u8], timeout: Option<f64>) -> PyResult<Vec<u8>> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.call(body, timeout).map_err(bus_err)
    }
}

#[pyclass(name = "ActionClient", unsendable)]
struct PyNodeActionClient {
    inner: RustNodeActionClient,
}

#[pymethods]
impl PyNodeActionClient {
    #[getter]
    fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    /// Send a goal and collect FEEDBACK/RESULT messages.
    ///
    /// Returns `list[dict]` with keys `kind`, `body`, `goal_id`, `action_name`.
    /// `timeout` is seconds.
    #[pyo3(signature = (body, goal_id=None, timeout=None))]
    fn send_goal(
        &self,
        py: Python<'_>,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<f64>,
    ) -> PyResult<PyObject> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let messages = self
            .inner
            .send_goal(body, goal_id, timeout)
            .map_err(bus_err)?;
        let list = pyo3::types::PyList::empty(py);
        for msg in messages {
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
            list.append(dict)?;
        }
        Ok(list.into())
    }

    /// Cancel a goal; returns the RESULT message dict. `timeout` is seconds.
    #[pyo3(signature = (goal_id, body=None, timeout=None))]
    fn cancel(
        &self,
        py: Python<'_>,
        goal_id: &str,
        body: Option<&[u8]>,
        timeout: Option<f64>,
    ) -> PyResult<PyObject> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let msg = self
            .inner
            .cancel(goal_id, body.unwrap_or(b""), timeout)
            .map_err(bus_err)?;
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
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        );
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
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        );
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
        service_frontend=None,
        service_backend=None,
        action_backend=None,
        action_frontend=None,
    ))]
    fn new(
        name: String,
        host: &str,
        transport: &str,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Self {
        Self {
            inner: RustNode::with_options(
                name,
                py_node_options(
                    host,
                    transport,
                    message_xsub,
                    message_xpub,
                    service_frontend,
                    service_backend,
                    action_backend,
                    action_frontend,
                ),
            ),
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
            inner: self.inner.create_client_raw(service_name).map_err(bus_err)?,
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
    ))]
    fn start(
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

        let broker = RustRobotBusBroker::start(config).map_err(anyhow_err)?;
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

    println!("robot-bus-broker starting message + service + action buses + gRPC…");
    let broker = RustRobotBusBroker::start(config).map_err(anyhow_err)?;
    let mut broker = PyRobotBusBroker {
        inner: Some(broker),
    };
    #[cfg(feature = "grpc")]
    println!(
        "gRPC / gRPC-Web listening on http://{}",
        broker.grpc_listen()?
    );

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
    m.add_class::<PyNodeServiceClient>()?;
    m.add_class::<PyNodeActionClient>()?;
    m.add_class::<PyShutdownHandle>()?;
    m.add_class::<PyTimerHandle>()?;
    m.add_class::<PyRobotBusBroker>()?;
    m.add_function(wrap_pyfunction!(message_xsub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(message_xpub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(run_broker, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
