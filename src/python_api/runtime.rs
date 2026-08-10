//! Context, callback groups, timers, and executors.

use std::time::Duration;

use pyo3::prelude::*;

use crate::runtime::{
    CallbackGroup, CallbackGroupType, Context as RustContext,
    MultiThreadedExecutor as RustMultiThreadedExecutor,
    ShutdownHandle as RustShutdownHandle, SingleThreadedExecutor as RustSingleThreadedExecutor,
    TimerHandle as RustTimerHandle,
};

use super::node::PyNode;
use super::util::{bus_err, py_node_options};

#[pyclass(name = "ShutdownHandle")]
#[derive(Clone)]
pub(crate) struct PyShutdownHandle {
    pub(crate) inner: RustShutdownHandle,
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
pub(crate) struct PyTimerHandle {
    pub(crate) inner: RustTimerHandle,
}

#[pyclass(name = "CallbackGroupType", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PyCallbackGroupType {
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
pub(crate) struct PyCallbackGroup {
    pub(crate) inner: CallbackGroup,
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

#[pyclass(name = "Context")]
#[derive(Clone)]
pub(crate) struct PyContext {
    pub(crate) inner: RustContext,
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
pub(crate) struct PySingleThreadedExecutor {
    pub(crate) inner: RustSingleThreadedExecutor,
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
pub(crate) struct PyMultiThreadedExecutor {
    pub(crate) inner: RustMultiThreadedExecutor,
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
