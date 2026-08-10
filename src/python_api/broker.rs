//! In-process broker and CLI entrypoint.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::broker::{
    RobotBusBroker as RustRobotBusBroker, RobotBusConfig, apply_federation_opts,
    parse_robot_bus_config, robot_bus_broker_help,
};
use crate::shutdown;

use super::runtime::PyContext;
use super::util::anyhow_err;

/// In-process broker: message + service + action buses + gRPC on background threads.
#[pyclass(name = "RobotBusBroker", unsendable)]
pub(crate) struct PyRobotBusBroker {
    pub(crate) inner: Option<RustRobotBusBroker>,
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
        api_listen = None,
        peers = None,
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
        api_listen: Option<String>,
        peers: Option<Vec<String>>,
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
            if let Some(v) = cors_origins {
                config.grpc.cors_origins = v;
            }
        }
        #[cfg(not(feature = "grpc"))]
        {
            let _ = cors_origins;
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

        if let Some(peer_list) = peers {
            crate::broker::apply_api_peers(&mut config, &peer_list).map_err(anyhow_err)?;
        }

        if no_discovery {
            config.discovery.enabled = false;
        }
        config.discovery.domain_id = domain_id;
        if let Some(v) = advertise_host {
            if !v.is_empty() {
                config.discovery.advertise_host = Some(v);
            }
        }
        let listen = api_listen.or(grpc_listen);
        if let Some(v) = listen {
            if !v.is_empty() {
                #[cfg(feature = "grpc")]
                {
                    config.grpc.listen = v
                        .parse()
                        .map_err(|e| PyRuntimeError::new_err(format!("invalid api_listen: {e}")))?;
                    #[cfg(feature = "console")]
                    {
                        config.console.listen = config.grpc.listen;
                    }
                }
                #[cfg(not(feature = "grpc"))]
                {
                    let _ = v;
                }
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

pub(crate) fn normalize_bind(addr: &str) -> String {
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

pub(crate) fn print_broker_help() {
    print!("{}", robot_bus_broker_help());
}

/// Blocking CLI entry: start broker and wait for Ctrl+C (or Unix SIGTERM).
///
/// Used by the `robot-bus-broker` console script after `pip install robot-bus`.
#[pyfunction]
pub(crate) fn run_broker(py: Python<'_>) -> PyResult<()> {
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
        "gRPC + WebSocket listening on http://{}",
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
