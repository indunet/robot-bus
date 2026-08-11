//! Shared helpers for the Python extension.

use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::action_bus::ActionKind;
use crate::discovery::{DiscoverOpts as RustDiscoverOpts, wait as discover_wait};
use crate::errors::BusError;
use crate::runtime::{NodeOptions as RustNodeOptions, ParameterValue};
use crate::transports;

pub(crate) fn bus_err(err: BusError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

pub(crate) fn parameter_value_from_py(value: &Bound<'_, PyAny>) -> PyResult<ParameterValue> {
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

pub(crate) fn parameter_value_to_py(py: Python<'_>, value: ParameterValue) -> PyResult<PyObject> {
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
pub(crate) unsafe fn allow_threads_io<'py, T, R>(
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

pub(crate) fn anyhow_err(err: anyhow::Error) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

pub(crate) fn map_endpoint_err(err: String) -> PyErr {
    PyRuntimeError::new_err(err)
}

pub(crate) fn action_message_to_py(
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

pub(crate) fn py_node_options(
    host: &str,
    transport: &str,
    ws_url: Option<String>,
    message_xsub: Option<String>,
    message_xpub: Option<String>,
    service_frontend: Option<String>,
    service_backend: Option<String>,
    action_backend: Option<String>,
    action_frontend: Option<String>,
) -> PyResult<crate::runtime::NodeOptions> {
    if transport == "ws" {
        #[cfg(feature = "ws")]
        {
            return Ok(match ws_url {
                Some(url) => RustNodeOptions::ws_at(url),
                None => RustNodeOptions::ws(),
            });
        }
        #[cfg(not(feature = "ws"))]
        {
            let _ = ws_url;
            return Err(PyRuntimeError::new_err(
                "transport=\"grpc\" requires the grpc feature",
            ));
        }
    }
    if ws_url.is_some() {
        return Err(PyRuntimeError::new_err(
            "ws_url is only valid when transport=\"grpc\"",
        ));
    }
    Ok(crate::runtime::NodeOptions {
        host: host.into(),
        transport: transport.into(),
        ws_url: None,
        console_url: None,
        message_xsub,
        message_xpub,
        service_frontend,
        service_backend,
        action_backend,
        action_frontend,
    })
}

pub(crate) fn py_discover_options(
    transport: &str,
    api_url: Option<&str>,
    broker_id: Option<String>,
    timeout: f64,
) -> PyResult<RustNodeOptions> {
    let base = match transport {
        "tcp" => RustNodeOptions::tcp(),
        "ipc" => RustNodeOptions::ipc(),
        "inproc" => RustNodeOptions::inproc(),
        "ws" => {
            #[cfg(feature = "ws")]
            {
                RustNodeOptions::ws()
            }
            #[cfg(not(feature = "ws"))]
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
        broker_id: broker_id.filter(|s| !s.is_empty()),
        ..Default::default()
    };
    if let Some(url) = api_url {
        if !url.is_empty() {
            opts.api_url = url.to_string();
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
pub(crate) fn message_xsub_endpoint(host: &str, transport: &str) -> PyResult<String> {
    transports::message_xsub_endpoint(host, transport).map_err(map_endpoint_err)
}

#[pyfunction]
#[pyo3(signature = (host="localhost", transport="tcp"))]
pub(crate) fn message_xpub_endpoint(host: &str, transport: &str) -> PyResult<String> {
    transports::message_xpub_endpoint(host, transport).map_err(map_endpoint_err)
}

#[pyfunction]
pub(crate) fn ros2_available() -> bool {
    // C++/Python bridges are native; this FFI flag stays false.
    // Python `robot_bus.ros2_available()` checks rclpy instead.
    false
}
