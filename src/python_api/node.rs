//! Python Node binding.

use std::sync::Arc;
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};

use crate::runtime::{
    ActionGoalHandler, Node as RustNode, NodeOptions as RustNodeOptions, QosProfile,
};

use super::clients::{PyNodeActionClient, PyNodeServiceClient};
use super::pub_sub::PyTopicPublisher;
use super::runtime::{
    PyActionServerHandle, PyCallbackGroup, PyCallbackGroupType, PyContext, PyServiceHandle,
    PyShutdownHandle, PySubscriptionHandle, PyTimerHandle,
};
use super::util::{
    bus_err, parameter_to_py, parameter_value_from_py, py_discover_options, py_node_options,
};

#[pyclass(name = "Node", unsendable)]
pub(crate) struct PyNode {
    pub(crate) inner: RustNode,
}

#[pymethods]
impl PyNode {
    #[new]
    #[pyo3(signature = (
        name,
        host="localhost",
        transport="tcp",
        ws_url=None,
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
        ws_url: Option<String>,
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

    /// ROS 2–style preferred entry: share `context`, connect local broker over TCP.
    #[classmethod]
    fn with_context(_cls: &Bound<'_, PyType>, context: &PyContext, name: String) -> Self {
        Self {
            inner: RustNode::with_context(&context.inner, name),
        }
    }

    /// Like [`with_context`] with explicit host (TCP).
    #[classmethod]
    #[pyo3(signature = (context, name, host="localhost"))]
    fn with_context_at(
        _cls: &Bound<'_, PyType>,
        context: &PyContext,
        name: String,
        host: &str,
    ) -> Self {
        Self {
            inner: RustNode::with_context_options(
                &context.inner,
                name,
                RustNodeOptions::tcp_at(host),
            ),
        }
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
                Some(p) => RustNode::inproc_at_with_context(&context.inner, name, p),
                None => RustNode::inproc_with_context(&context.inner, name),
            },
        }
    }

    /// WebSocket RPC client node talking to the local broker gateway (`http://127.0.0.1:15570`).
    #[cfg(feature = "ws")]
    #[classmethod]
    #[pyo3(signature = (name,))]
    fn ws(_cls: &Bound<'_, PyType>, name: String) -> Self {
        Self {
            inner: RustNode::ws(name),
        }
    }

    /// Discover a broker via HTTP `GET /api/v1/discover`, then connect with `transport`.
    ///
    /// Transport is still chosen by the caller (`tcp` / `ipc` / `inproc` / `ws`).
    /// Discovery only fills host / paths / gateway URL.
    #[classmethod]
    #[pyo3(signature = (
        name,
        transport="tcp",
        *,
        api_url=None,
        broker_id=None,
        timeout=3.0,
    ))]
    fn discover(
        _cls: &Bound<'_, PyType>,
        name: String,
        transport: &str,
        api_url: Option<&str>,
        broker_id: Option<String>,
        timeout: f64,
    ) -> PyResult<Self> {
        let options = py_discover_options(transport, api_url, broker_id, timeout)?;
        Ok(Self {
            inner: RustNode::with_options(name, options),
        })
    }

    /// WebSocket RPC client node talking to `url` (e.g. `http://127.0.0.1:15570`).
    #[cfg(feature = "ws")]
    #[classmethod]
    #[pyo3(signature = (name, url))]
    fn ws_at(_cls: &Bound<'_, PyType>, name: String, url: &str) -> Self {
        Self {
            inner: RustNode::ws_at(name, url),
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    /// Broker link state: `created` / `discovering` / `connecting` / `connected` /
    /// `reconnecting` / `shutdown`.
    #[getter]
    fn connection_state(&self) -> &'static str {
        self.inner.connection_state().as_str()
    }

    /// Block until the node is connected to the broker. `timeout` is seconds;
    /// `None` waits until connected or shutdown. Returns `False` on timeout.
    #[pyo3(signature = (timeout=None))]
    fn wait_for_broker(&self, timeout: Option<f64>) -> bool {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.wait_for_broker(timeout)
    }

    /// `callback(old: str, new: str, reason: str)` on the session thread.
    fn add_on_connection_event(&self, callback: Py<PyAny>) {
        self.inner.add_on_connection_event(move |old, new, reason| {
            Python::with_gil(|py| {
                let _ = callback.call1(py, (old.as_str(), new.as_str(), reason));
            });
        });
    }

    /// Declare a local parameter (`bool` / `int` / `float` / `str`).
    /// Returns a dict `{"name", "value"}` (ROS 2 returns a Parameter).
    fn declare_parameter(
        &mut self,
        py: Python<'_>,
        name: &str,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let param = self
            .inner
            .declare_parameter(name, parameter_value_from_py(value)?)
            .map_err(bus_err)?;
        parameter_to_py(py, param)
    }

    /// Read a previously declared parameter.
    /// Returns a dict `{"name", "value"}` (ROS 2 `Parameter`; use `["value"]` like `.value`).
    fn get_parameter(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        let param = self.inner.get_parameter(name).map_err(bus_err)?;
        parameter_to_py(py, param)
    }

    /// Update a declared parameter (type must match).
    fn set_parameter(&mut self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner
            .set_parameter(crate::runtime::Parameter::new(
                name,
                parameter_value_from_py(value)?,
            ))
            .map_err(bus_err)
    }

    fn has_parameter(&self, name: &str) -> bool {
        self.inner.has_parameter(name)
    }

    /// Remove a declared parameter.
    fn undeclare_parameter(&mut self, name: &str) -> PyResult<()> {
        self.inner.undeclare_parameter(name).map_err(bus_err)
    }

    /// List parameter names (ROS 2 `list_parameters`).
    ///
    /// Returns `{"names": [...], "prefixes": [...]}`. Omit args to list all recursively.
    #[pyo3(signature = (prefixes=None, depth=0))]
    fn list_parameters(
        &self,
        py: Python<'_>,
        prefixes: Option<Vec<String>>,
        depth: u64,
    ) -> PyResult<PyObject> {
        let owned = prefixes.unwrap_or_default();
        let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        let result = self.inner.list_parameters(&refs, depth);
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("names", result.names)?;
        dict.set_item("prefixes", result.prefixes)?;
        Ok(dict.into())
    }

    /// All declared parameters as `list[dict]` with keys `name`, `value`.
    fn list_all_parameters(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = pyo3::types::PyList::empty(py);
        for param in self.inner.list_all_parameters() {
            list.append(parameter_to_py(py, param)?)?;
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

    #[pyo3(signature = (topic, qos_depth=None))]
    fn create_publisher(
        &mut self,
        topic: &str,
        qos_depth: Option<i32>,
    ) -> PyResult<PyTopicPublisher> {
        let inner = match qos_depth.filter(|d| *d > 0) {
            Some(depth) => self
                .inner
                .create_publisher_raw_with_qos(topic, QosProfile::keep_last(depth)),
            None => self.inner.create_publisher_raw(topic),
        }
        .map_err(bus_err)?;
        Ok(PyTopicPublisher { inner })
    }

    /// Register a subscription callback `callback(payload: bytes)`.
    #[pyo3(signature = (topic, callback, callback_group=None, qos_depth=None))]
    fn create_subscription(
        slf: &Bound<'_, Self>,
        topic: &str,
        callback: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
        qos_depth: Option<i32>,
    ) -> PyResult<PySubscriptionHandle> {
        let handle = {
            let mut this = slf.borrow_mut();
            let cb: crate::runtime::MessageCallback = Arc::new(move |payload| {
                Python::with_gil(|py| {
                    let payload = PyBytes::new(py, payload);
                    if let Err(err) = callback.call1(py, (payload,)) {
                        err.print(py);
                    }
                });
            });
            let group = callback_group.map(|g| &g.inner);
            match qos_depth.filter(|d| *d > 0) {
                Some(depth) => this.inner.create_subscription_raw_with_qos(
                    topic,
                    QosProfile::keep_last(depth),
                    cb,
                    group,
                ),
                None => this.inner.create_subscription_raw(topic, cb, group),
            }
            .map_err(bus_err)?
        };
        Ok(PySubscriptionHandle::new(slf.clone().unbind(), handle))
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

    /// Alias for [`create_timer`](Self::create_timer) (ROS 2 `create_wall_timer`).
    #[pyo3(signature = (period, callback, callback_group=None))]
    fn create_wall_timer(
        &mut self,
        period: f64,
        callback: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
    ) -> PyResult<PyTimerHandle> {
        self.create_timer(period, callback, callback_group)
    }

    fn cancel_timer(&mut self, handle: PyTimerHandle) -> PyResult<()> {
        self.inner.cancel_timer(handle.inner).map_err(bus_err)
    }

    /// Register a service server.
    ///
    /// `handler(body: bytes) -> bytes`
    #[pyo3(signature = (service_name, handler, callback_group=None, qos_depth=None))]
    fn create_service(
        slf: &Bound<'_, Self>,
        service_name: &str,
        handler: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
        qos_depth: Option<i32>,
    ) -> PyResult<PyServiceHandle> {
        let handle = {
            let mut this = slf.borrow_mut();
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
            match qos_depth.filter(|d| *d > 0) {
                Some(depth) => this.inner.create_service_raw_with_qos(
                    service_name,
                    QosProfile::keep_last(depth),
                    cb,
                    group,
                ),
                None => this.inner.create_service_raw(service_name, cb, group),
            }
            .map_err(bus_err)?
        };
        Ok(PyServiceHandle::new(slf.clone().unbind(), handle))
    }

    /// Create a service client bound to `service_name` (ROS 2 `create_client`).
    #[pyo3(signature = (service_name, qos_depth=None))]
    fn create_client(
        &mut self,
        service_name: &str,
        qos_depth: Option<i32>,
    ) -> PyResult<PyNodeServiceClient> {
        let inner = match qos_depth.filter(|d| *d > 0) {
            Some(depth) => self
                .inner
                .create_client_raw_with_qos(service_name, QosProfile::keep_last(depth)),
            None => self.inner.create_client_raw(service_name),
        }
        .map_err(bus_err)?;
        Ok(PyNodeServiceClient { inner })
    }

    /// Register an action server (ROS 2–style `create_action_server`).
    ///
    /// `handler(payload: bytes) -> list[tuple[str, bytes]]`
    /// where each tuple is `(phase, body)` and `phase` is typically `"FEEDBACK"` / `"RESULT"`.
    #[pyo3(signature = (action_name, handler, callback_group=None, qos_depth=None))]
    fn create_action_server(
        slf: &Bound<'_, Self>,
        action_name: &str,
        handler: Py<PyAny>,
        callback_group: Option<&PyCallbackGroup>,
        qos_depth: Option<i32>,
    ) -> PyResult<PyActionServerHandle> {
        let handle = {
            let mut this = slf.borrow_mut();
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
            match qos_depth.filter(|d| *d > 0) {
                Some(depth) => this.inner.create_action_server_raw_with_qos(
                    action_name,
                    QosProfile::keep_last(depth),
                    cb,
                    group,
                ),
                None => this.inner.create_action_server_raw(action_name, cb, group),
            }
            .map_err(bus_err)?
        };
        Ok(PyActionServerHandle::new(slf.clone().unbind(), handle))
    }

    /// Create an action client bound to `action_name` (ROS 2 `create_action_client`).
    #[pyo3(signature = (action_name, qos_depth=None))]
    fn create_action_client(
        &mut self,
        action_name: &str,
        qos_depth: Option<i32>,
    ) -> PyResult<PyNodeActionClient> {
        let inner = match qos_depth.filter(|d| *d > 0) {
            Some(depth) => self
                .inner
                .create_action_client_raw_with_qos(action_name, QosProfile::keep_last(depth)),
            None => self.inner.create_action_client_raw(action_name),
        }
        .map_err(bus_err)?;
        Ok(PyNodeActionClient { inner })
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

    /// Wait for one message on `topic`. Returns payload bytes or `None` on timeout.
    #[pyo3(signature = (topic, timeout=None))]
    fn wait_for_message(&mut self, topic: &str, timeout: Option<f64>) -> PyResult<Option<Vec<u8>>> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.wait_for_message(topic, timeout).map_err(bus_err)
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
