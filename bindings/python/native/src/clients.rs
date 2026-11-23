//! Service / action client bindings.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use pyo3::exceptions::{PyRuntimeError, PyTimeoutError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::runtime::{
    NodeActionClientRaw as RustNodeActionClient, NodeServiceClientRaw as RustNodeServiceClient,
    RawActionFeedbackCallback, RawGoalHandle as RustRawGoalHandle,
};

use super::util::{action_message_to_py, allow_threads_io, bus_err};

#[pyclass(name = "ServiceClient")]
pub(crate) struct PyNodeServiceClient {
    pub(crate) inner: RustNodeServiceClient,
}

#[pymethods]
impl PyNodeServiceClient {
    #[getter]
    fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    fn service_is_ready(&self) -> bool {
        self.inner.service_is_ready()
    }

    /// Poll until the service has workers. `timeout` is seconds; `None` waits forever.
    #[pyo3(signature = (timeout=None))]
    fn wait_for_service(&self, timeout: Option<f64>) -> bool {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.wait_for_service(timeout)
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

#[pyclass(name = "ActionClient")]
pub(crate) struct PyNodeActionClient {
    pub(crate) inner: RustNodeActionClient,
}

#[pyclass(name = "ActionGoalHandle")]
pub(crate) struct PyActionGoalHandle {
    pub(crate) inner: RustRawGoalHandle,
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

    fn action_server_is_ready(&self) -> bool {
        self.inner.action_server_is_ready()
    }

    /// Poll until the action server has workers. `timeout` is seconds; `None` waits forever.
    #[pyo3(signature = (timeout=None))]
    fn wait_for_action_server(&self, timeout: Option<f64>) -> bool {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.wait_for_action_server(timeout)
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
