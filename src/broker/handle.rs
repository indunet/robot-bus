//! In-process broker handle: start all buses (and the WebSocket RPC API) on background threads.
//!
//! Prefer [`RobotBusBroker::start`] over the CLI binary when embedding in application code.
//! `start` uses per-bus [`run_with_shutdown`](super::service_bus::run_with_shutdown) and does
//! **not** install a process-wide Ctrl+C handler (unlike the blocking `run` helpers).

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use zmq::Context as ZmqContext;

use super::action_bus::{self, ActionBusConfig, ActionMetrics};
use super::message_bus::{self, BusConfig, MessageMetrics};
use super::service_bus::{self, ServiceBusConfig, ServiceMetrics};
use crate::discovery::{
    DiscoverResponse, DiscoveryConfig, resolve_advertise_host, rewrite_bind_host,
};
use crate::runtime::Context as BusContext;
use crate::transports::{
    ACTION_BACKEND_CHANNEL, ACTION_FRONTEND_CHANNEL, BindAllOpts, SERVICE_BACKEND_CHANNEL,
    SERVICE_FRONTEND_CHANNEL, XPUB_CHANNEL, XSUB_CHANNEL, ipc_endpoint_in,
    inproc_endpoint_with_prefix,
};

#[cfg(feature = "console")]
use crate::tank::{TankEndpoints, TankManager};
#[cfg(all(feature = "console", not(feature = "ws")))]
use crate::console::serve_with_shutdown as serve_console_with_shutdown;
#[cfg(feature = "console")]
use crate::console::{BrokerEndpoints, ConsoleState, ControlPlaneHandle, StatusPublisherHandle};
#[cfg(feature = "ws")]
use crate::ws_gateway::{GatewayConfig, serve_on_listener};
use std::net::SocketAddr;

const STARTUP_SETTLE: Duration = Duration::from_millis(50);

#[cfg(feature = "ws")]
fn connect_url_for_listen(listen: SocketAddr) -> String {
    let host = match listen.ip() {
        std::net::IpAddr::V4(ip) if ip.is_unspecified() => "127.0.0.1".to_string(),
        std::net::IpAddr::V6(ip) if ip.is_unspecified() => "[::1]".to_string(),
        std::net::IpAddr::V6(ip) => format!("[{ip}]"),
        ip => ip.to_string(),
    };
    format!("http://{host}:{}", listen.port())
}

fn join_broker_thread(name: &str, handle: JoinHandle<Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|e| anyhow!("{name} thread panicked: {e:?}"))?
        .with_context(|| format!("{name} exited with error"))
}

/// Wait for a bus thread to publish its resolved TCP binds (`:0` → real ports).
///
/// On disconnect, join the thread so a bind `EADDRINUSE` (or panic) is not
/// flattened into a generic timeout error.
fn recv_bound_endpoints(
    name: &str,
    bound_rx: Receiver<(String, String)>,
    handle: JoinHandle<Result<()>>,
) -> Result<((String, String), JoinHandle<Result<()>>)> {
    match bound_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(pair) => Ok((pair, handle)),
        Err(RecvTimeoutError::Timeout) => Err(anyhow!(
            "{name} failed to report bound endpoints: timed out after 5s"
        )),
        Err(RecvTimeoutError::Disconnected) => {
            let detail = match handle.join() {
                Ok(Ok(())) => "thread exited without reporting binds".to_string(),
                Ok(Err(err)) => format!("{err:#}"),
                Err(panic) => format!("thread panicked: {panic:?}"),
            };
            Err(anyhow!(
                "{name} failed to report bound endpoints: {detail}"
            ))
        }
    }
}

/// Background handle for the message bus (XSUB/XPUB proxy).
pub struct MessageBusBroker {
    pub xsub_bind: String,
    pub xpub_bind: String,
    /// Per-topic counters updated by the proxy (shared with the console when enabled).
    pub metrics: Arc<MessageMetrics>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl MessageBusBroker {
    /// Bind and run the message bus on a background thread.
    ///
    /// Pass `Some(metrics)` only when the console (or another observer) needs
    /// topic counters — that enables libzmq capture. `None` keeps the forward
    /// path as a plain `proxy_steerable` with no capture overhead.
    pub(crate) fn start_with_zmq(
        zmq: ZmqContext,
        config: BusConfig,
        metrics: Option<Arc<MessageMetrics>>,
    ) -> Result<Self> {
        // Always keep an Arc for `self.metrics` (console may read zeros if unused).
        let stored = metrics.clone().unwrap_or_else(MessageMetrics::new);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let (bound_tx, bound_rx) = std::sync::mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            message_bus::run_with_shutdown_ctx_bound(
                zmq,
                config,
                shutdown_flag,
                metrics,
                Some(bound_tx),
            )
        });
        let ((xsub_bind, xpub_bind), handle) =
            recv_bound_endpoints("message bus", bound_rx, handle)?;
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            xsub_bind,
            xpub_bind,
            metrics: stored,
            shutdown,
            handle: Some(handle),
        })
    }

    /// Signal shutdown and join the broker thread.
    pub(crate) fn stop(mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            join_broker_thread("message_bus", handle)?;
        }
        Ok(())
    }
}

impl Drop for MessageBusBroker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Background handle for the service bus (dual-ROUTER).
pub struct ServiceBusBroker {
    pub frontend_bind: String,
    pub backend_bind: String,
    pub metrics: Arc<ServiceMetrics>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ServiceBusBroker {
    /// Bind and run the service bus on a background thread.
    pub(crate) fn start_with_zmq(
        zmq: ZmqContext,
        config: ServiceBusConfig,
        metrics: Option<Arc<ServiceMetrics>>,
    ) -> Result<Self> {
        let stored = metrics.clone().unwrap_or_else(ServiceMetrics::new);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let (bound_tx, bound_rx) = std::sync::mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            service_bus::run_with_shutdown_ctx_bound(
                zmq,
                config,
                shutdown_flag,
                metrics,
                Some(bound_tx),
            )
        });
        let ((frontend_bind, backend_bind), handle) =
            recv_bound_endpoints("service bus", bound_rx, handle)?;
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            frontend_bind,
            backend_bind,
            metrics: stored,
            shutdown,
            handle: Some(handle),
        })
    }

    /// Signal shutdown and join the broker thread.
    pub(crate) fn stop(mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            join_broker_thread("service_bus", handle)?;
        }
        Ok(())
    }
}

impl Drop for ServiceBusBroker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Background handle for the action bus (dual-ROUTER).
pub struct ActionBusBroker {
    pub frontend_bind: String,
    pub backend_bind: String,
    pub metrics: Arc<ActionMetrics>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl ActionBusBroker {
    /// Bind and run the action bus on a background thread.
    pub(crate) fn start_with_zmq(
        zmq: ZmqContext,
        config: ActionBusConfig,
        metrics: Option<Arc<ActionMetrics>>,
    ) -> Result<Self> {
        let stored = metrics.clone().unwrap_or_else(ActionMetrics::new);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let (bound_tx, bound_rx) = std::sync::mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            action_bus::run_with_shutdown_ctx_bound(
                zmq,
                config,
                shutdown_flag,
                metrics,
                Some(bound_tx),
            )
        });
        let ((frontend_bind, backend_bind), handle) =
            recv_bound_endpoints("action bus", bound_rx, handle)?;
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            frontend_bind,
            backend_bind,
            metrics: stored,
            shutdown,
            handle: Some(handle),
        })
    }

    /// Signal shutdown and join the broker thread.
    pub(crate) fn stop(mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            join_broker_thread("action_bus", handle)?;
        }
        Ok(())
    }
}

impl Drop for ActionBusBroker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// WebSocket RPC listen options (feature `ws`, enabled by default).
#[cfg(feature = "ws")]
#[derive(Clone, Debug)]
pub struct WsGatewayConfig {
    pub listen: SocketAddr,
    /// When empty, allow any origin (local-dev default).
    pub cors_origins: Vec<String>,
}

#[cfg(feature = "ws")]
impl Default for WsGatewayConfig {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:15570".parse().expect("default API listen"),
            cors_origins: Vec::new(),
        }
    }
}

/// Embedded Web console HTTP options (feature `console`, enabled by default).
///
/// When the `ws` feature is also enabled, the console UI + REST API are served
/// on [`WsGatewayConfig::listen`] instead — WebSocket RPC (`/ws`) and the console
/// share one port. `listen` here only takes effect when `ws` is disabled (or
/// this crate is built console-only).
#[cfg(feature = "console")]
#[derive(Clone, Debug)]
pub struct ConsoleBrokerConfig {
    /// When false, the console is not started.
    pub enabled: bool,
    /// When false, the in-console tank demo is hidden and cannot be started
    /// (`--no-tank`). Default true for local / sim use.
    pub tank_enabled: bool,
    /// Listen address used only when the `grpc` feature is disabled.
    pub listen: SocketAddr,
    /// Explicit CORS allowlist for cross-origin browser clients.
    /// Empty (default) disables CORS headers. Never uses `*`.
    pub cors_origins: Vec<String>,
}

#[cfg(feature = "console")]
impl Default for ConsoleBrokerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tank_enabled: true,
            listen: "0.0.0.0:15570".parse().expect("default console listen"),
            cors_origins: Vec::new(),
        }
    }
}

/// Configuration for starting all buses (and gRPC / console) in one process.
#[derive(Clone, Debug, Default)]
pub struct RobotBusConfig {
    pub message: BusConfig,
    pub service: ServiceBusConfig,
    pub action: ActionBusConfig,
    pub discovery: DiscoveryConfig,
    #[cfg(feature = "ws")]
    pub ws: WsGatewayConfig,
    #[cfg(feature = "console")]
    pub console: ConsoleBrokerConfig,
}

#[cfg(feature = "ws")]
struct WsGatewayHandle {
    pub listen: SocketAddr,
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    handle: Option<JoinHandle<Result<()>>>,
}

#[cfg(feature = "ws")]
impl WsGatewayHandle {
    fn start(mut config: GatewayConfig) -> Result<Self> {
        let requested = config.listen;
        // Bind on the calling thread so EADDRINUSE fails at start(), not only at stop().
        let std_listener = std::net::TcpListener::bind(requested)
            .with_context(|| format!("bind API listen {requested}"))?;
        std_listener
            .set_nonblocking(true)
            .context("set API listener nonblocking")?;
        let listen = std_listener
            .local_addr()
            .context("API listener local_addr")?;
        config.listen = listen;
        if let Some(disc) = config.discover.as_mut() {
            let mut d = (**disc).clone();
            d.api_url = connect_url_for_listen(listen);
            #[cfg(feature = "console")]
            if d.console_url.is_some() {
                d.console_url = Some(d.api_url.clone());
            }
            *disc = Arc::new(d);
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("create tokio runtime for WebSocket RPC gateway")?;
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(std_listener)
                    .context("tokio API listener")?;
                // `wait_for` resolves immediately if the watch already holds `true` (e.g.
                // the caller stopped right after start()); the old `while !*borrow() { changed().await }`
                // loop could instead park forever waiting for a *change* that already happened.
                let mut graceful_rx = shutdown_rx.clone();
                let graceful = async move {
                    let _ = graceful_rx.wait_for(|shutdown| *shutdown).await;
                };
                let mut force_rx = shutdown_rx;
                let force = async move {
                    let _ = force_rx.wait_for(|shutdown| *shutdown).await;
                    // Open client streams can block tonic's graceful drain forever;
                    // after a short grace period, drop the server future and free the port.
                    tokio::time::sleep(Duration::from_millis(500)).await;
                };
                tokio::select! {
                    result = serve_on_listener(config, listener, graceful) => result,
                    _ = force => Ok(()),
                }
            })
        });
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            listen,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        })
    }

    fn stop(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            join_broker_thread("grpc_gateway", handle)?;
        }
        Ok(())
    }
}

#[cfg(feature = "ws")]
impl Drop for WsGatewayHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Console-only HTTP server (no `grpc` feature) — otherwise the console shares
/// the gRPC gateway's listener (see [`GatewayConfig::console`]).
#[cfg(all(feature = "console", not(feature = "ws")))]
struct ConsoleHttpHandle {
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    handle: Option<JoinHandle<Result<()>>>,
}

#[cfg(all(feature = "console", not(feature = "ws")))]
impl ConsoleHttpHandle {
    fn start(
        listen: SocketAddr,
        state: Arc<ConsoleState>,
        cors_origins: Vec<String>,
    ) -> Result<Self> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("create tokio runtime for console HTTP")?;
            rt.block_on(async move {
                let shutdown = async move {
                    while !*shutdown_rx.borrow() {
                        if shutdown_rx.changed().await.is_err() {
                            break;
                        }
                    }
                };
                serve_console_with_shutdown(listen, state, cors_origins, shutdown).await
            })
        });
        thread::sleep(STARTUP_SETTLE);
        Ok(Self {
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        })
    }

    fn stop(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            join_broker_thread("console_http", handle)?;
        }
        Ok(())
    }
}

#[cfg(all(feature = "console", not(feature = "ws")))]
impl Drop for ConsoleHttpHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Rewrite a bind address into a client connect endpoint (`0.0.0.0` / `*` → `127.0.0.1`).
fn bind_to_connect(bind: &str) -> String {
    if let Some(rest) = bind.strip_prefix("tcp://0.0.0.0:") {
        return format!("tcp://127.0.0.1:{rest}");
    }
    if let Some(rest) = bind.strip_prefix("tcp://*:") {
        return format!("tcp://127.0.0.1:{rest}");
    }
    bind.to_string()
}

/// Background handle that owns message + service + action brokers (and gRPC / console when enabled).
pub struct RobotBusBroker {
    pub message: MessageBusBroker,
    pub service: ServiceBusBroker,
    pub action: ActionBusBroker,
    #[cfg(feature = "ws")]
    ws: WsGatewayHandle,
    /// Console-only HTTP server; `None` when `grpc` is enabled (console shares its port)
    /// or the console is disabled.
    #[cfg(all(feature = "console", not(feature = "ws")))]
    console: Option<ConsoleHttpHandle>,
    #[cfg(feature = "console")]
    status_pub: Option<StatusPublisherHandle>,
    #[cfg(feature = "console")]
    control_plane: Option<ControlPlaneHandle>,
    #[cfg(feature = "console")]
    console_listen: Option<SocketAddr>,
    #[cfg(feature = "console")]
    tank: Option<Arc<TankManager>>,
    /// Snapshot served at `GET /api/v1/discover`.
    pub discover: DiscoverResponse,
}

impl RobotBusBroker {
    /// Start message, service, and action buses (and gRPC / console when enabled) on background threads.
    ///
    /// Creates a private ZeroMQ context. For same-process `inproc://`, use
    /// [`start_with_context`](Self::start_with_context) and share that context with Nodes.
    pub fn start(config: RobotBusConfig) -> Result<Self> {
        let context = BusContext::new();
        Self::start_with_context(&context, config)
    }

    /// Start buses using a shared [`crate::Context`] (required for inproc with SDK Nodes).
    ///
    /// Takes a shared reference; the ZMQ context is refcounted, so the caller keeps using
    /// the same `context` for Nodes without an explicit `.clone()`.
    pub fn start_with_context(context: &BusContext, mut config: RobotBusConfig) -> Result<Self> {
        let broker_id = normalize_broker_id(&mut config);
        let bind_opts = BindAllOpts::for_broker(&broker_id);
        config.message.bind_opts = bind_opts.clone();
        config.service.bind_opts = bind_opts.clone();
        config.action.bind_opts = bind_opts.clone();

        let advertise_host = resolve_advertise_host(&config.discovery);
        let zmq = context.clone_zmq();
        // Capture metrics only when the console will read them — otherwise keep
        // the message bus on a plain proxy_steerable with zero monitoring overhead.
        #[cfg(feature = "console")]
        let message_metrics = if config.console.enabled {
            Some(MessageMetrics::new())
        } else {
            None
        };
        #[cfg(not(feature = "console"))]
        let message_metrics = None;

        let message =
            MessageBusBroker::start_with_zmq(zmq.clone(), config.message.clone(), message_metrics)?;
        #[cfg(feature = "console")]
        let (service_metrics, action_metrics) = if config.console.enabled {
            (Some(ServiceMetrics::new()), Some(ActionMetrics::new()))
        } else {
            (None, None)
        };
        #[cfg(not(feature = "console"))]
        let (service_metrics, action_metrics) = (None, None);
        let service =
            ServiceBusBroker::start_with_zmq(zmq.clone(), config.service.clone(), service_metrics)?;
        let action = ActionBusBroker::start_with_zmq(zmq, config.action.clone(), action_metrics)?;

        let api_listen = {
            #[cfg(feature = "ws")]
            {
                config.ws.listen
            }
            #[cfg(all(not(feature = "ws"), feature = "console"))]
            {
                config.console.listen
            }
            #[cfg(all(not(feature = "ws"), not(feature = "console")))]
            {
                "0.0.0.0:15570"
                    .parse::<SocketAddr>()
                    .expect("default api listen")
            }
        };
        let api_url = format!("http://{advertise_host}:{}", api_listen.port());
        let msg_xsub = rewrite_bind_host(&message.xsub_bind, &advertise_host);
        let msg_xpub = rewrite_bind_host(&message.xpub_bind, &advertise_host);
        let svc_fe = rewrite_bind_host(&service.frontend_bind, &advertise_host);
        let svc_be = rewrite_bind_host(&service.backend_bind, &advertise_host);
        let act_fe = rewrite_bind_host(&action.frontend_bind, &advertise_host);
        let act_be = rewrite_bind_host(&action.backend_bind, &advertise_host);
        #[allow(unused_mut)] // mutated after WS bind when `ws` is enabled (`:0` → real port)
        let mut discover = DiscoverResponse {
            broker_id: broker_id.clone(),
            domain_id: config.discovery.domain_id,
            advertise_host: advertise_host.clone(),
            api_url: api_url.clone(),
            message_xsub: msg_xsub.clone(),
            message_xpub: msg_xpub.clone(),
            service_frontend: svc_fe.clone(),
            service_backend: svc_be.clone(),
            action_frontend: act_fe.clone(),
            action_backend: act_be.clone(),
            ipc_dir: config
                .message
                .bind_all_transports
                .then(|| bind_opts.ipc_dir.clone()),
            inproc_prefix: config
                .message
                .bind_all_transports
                .then(|| bind_opts.inproc_prefix.clone()),
            console_url: {
                #[cfg(feature = "console")]
                {
                    config.console.enabled.then(|| api_url.clone())
                }
                #[cfg(not(feature = "console"))]
                {
                    None
                }
            },
        };

        // Build console state before starting the gRPC gateway — when both features
        // are enabled, REST + static UI routes merge onto the same listener below.
        #[cfg(feature = "console")]
        let console_state: Option<Arc<ConsoleState>> = if config.console.enabled {
            let grpc_addr = {
                #[cfg(feature = "ws")]
                {
                    config.ws.listen.to_string()
                }
                #[cfg(not(feature = "ws"))]
                {
                    String::new()
                }
            };
            let web_addr = {
                #[cfg(feature = "ws")]
                {
                    config.ws.listen.to_string()
                }
                #[cfg(not(feature = "ws"))]
                {
                    config.console.listen.to_string()
                }
            };
            let endpoints = BrokerEndpoints {
                msg_xsub: message.xsub_bind.clone(),
                msg_xpub: message.xpub_bind.clone(),
                svc_fe: service.frontend_bind.clone(),
                svc_be: service.backend_bind.clone(),
                act_fe: action.frontend_bind.clone(),
                act_be: action.backend_bind.clone(),
                ws: grpc_addr,
                web: web_addr,
                discover: discover.clone(),
            };
            let tank = TankManager::new(TankEndpoints {
                message_xsub: bind_to_connect(&message.xsub_bind),
                message_xpub: bind_to_connect(&message.xpub_bind),
                service_frontend: bind_to_connect(&service.frontend_bind),
                service_backend: bind_to_connect(&service.backend_bind),
                action_backend: bind_to_connect(&action.backend_bind),
            });
            Some(ConsoleState::new(
                endpoints,
                message.metrics.clone(),
                service.metrics.clone(),
                action.metrics.clone(),
                tank,
                config.console.tank_enabled,
            ))
        } else {
            None
        };

        #[cfg(feature = "ws")]
        let ws = {
            let gateway = GatewayConfig {
                listen: config.ws.listen,
                message_xpub: bind_to_connect(&message.xpub_bind),
                message_xsub: bind_to_connect(&message.xsub_bind),
                service_frontend: bind_to_connect(&service.frontend_bind),
                action_frontend: bind_to_connect(&action.frontend_bind),
                cors_origins: config.ws.cors_origins.clone(),
                // When console is on, discover is served from console api_router.
                discover: {
                    #[cfg(feature = "console")]
                    {
                        if console_state.is_none() {
                            Some(Arc::new(discover.clone()))
                        } else {
                            None
                        }
                    }
                    #[cfg(not(feature = "console"))]
                    {
                        Some(Arc::new(discover.clone()))
                    }
                },
                #[cfg(feature = "console")]
                console: console_state.clone(),
            };
            WsGatewayHandle::start(gateway)?
        };
        #[cfg(feature = "ws")]
        {
            discover.api_url = connect_url_for_listen(ws.listen);
            #[cfg(feature = "console")]
            if discover.console_url.is_some() {
                discover.console_url = Some(discover.api_url.clone());
            }
        }

        // Console-only HTTP server — only needed when `grpc` is disabled; otherwise
        // the console shares the gRPC gateway's listener started above.
        #[cfg(all(feature = "console", not(feature = "ws")))]
        let console = match &console_state {
            Some(state) => Some(ConsoleHttpHandle::start(
                config.console.listen,
                state.clone(),
                config.console.cors_origins.clone(),
            )?),
            None => None,
        };

        // Control-plane subscriber + 1 Hz status publisher (feature `console`),
        // regardless of whether the console shares the gRPC port or has its own.
        #[cfg(feature = "console")]
        let (status_pub, control_plane) = match &console_state {
            Some(state) => {
                let control_plane = ControlPlaneHandle::start(
                    state.clone(),
                    bind_to_connect(&message.xpub_bind),
                    bind_to_connect(&service.backend_bind),
                )
                .context("start console control plane")?;
                let status_pub = StatusPublisherHandle::start(
                    state.clone(),
                    bind_to_connect(&message.xsub_bind),
                )
                .context("start console status publisher")?;
                (Some(status_pub), Some(control_plane))
            }
            None => (None, None),
        };

        #[cfg(feature = "console")]
        let console_listen: Option<SocketAddr> = if console_state.is_some() {
            #[cfg(feature = "ws")]
            {
                Some(ws.listen)
            }
            #[cfg(not(feature = "ws"))]
            {
                Some(config.console.listen)
            }
        } else {
            None
        };

        #[cfg(feature = "console")]
        let tank = console_state.as_ref().map(|s| Arc::clone(&s.tank));

        println!("{}", format_startup_banner(&discover));

        Ok(Self {
            message,
            service,
            action,
            #[cfg(feature = "ws")]
            ws,
            #[cfg(all(feature = "console", not(feature = "ws")))]
            console,
            #[cfg(feature = "console")]
            status_pub,
            #[cfg(feature = "console")]
            control_plane,
            #[cfg(feature = "console")]
            console_listen,
            #[cfg(feature = "console")]
            tank,
            discover,
        })
    }

    /// WebSocket RPC listen address (feature `ws`).
    #[cfg(feature = "ws")]
    pub fn api_listen(&self) -> SocketAddr {
        self.ws.listen
    }

    /// Base URL for WebSocket RPC clients (`http://127.0.0.1:port` when the broker binds `0.0.0.0`).
    #[cfg(feature = "ws")]
    pub fn api_url(&self) -> String {
        connect_url_for_listen(self.ws.listen)
    }

    /// Console HTTP listen address when the console is running (feature `console`).
    ///
    /// This is the API listen address when `ws` is also enabled (single port),
    /// or the console's own listen address otherwise.
    #[cfg(feature = "console")]
    pub fn console_listen(&self) -> Option<SocketAddr> {
        self.console_listen
    }

    /// Stop all buses (and gRPC / console) and join their threads.
    pub fn stop(self) -> Result<()> {
        // Ask the console background threads to wind down before tearing down
        // the gateway they publish/subscribe through; give them a moment to
        // notice before we start joining anything.
        #[cfg(feature = "console")]
        if let Some(tank) = self.tank.as_ref() {
            tank.shutdown();
        }
        #[cfg(feature = "console")]
        if let Some(status_pub) = self.status_pub.as_ref() {
            status_pub.request_stop();
        }
        #[cfg(feature = "console")]
        if let Some(control_plane) = self.control_plane.as_ref() {
            control_plane.request_stop();
        }
        #[cfg(feature = "console")]
        thread::sleep(Duration::from_millis(50));

        #[cfg(all(feature = "console", not(feature = "ws")))]
        let console = match self.console {
            Some(c) => c.stop(),
            None => Ok(()),
        };
        #[cfg(feature = "ws")]
        let ws_handle = self.ws.stop();

        #[cfg(feature = "console")]
        if let Some(status_pub) = self.status_pub {
            status_pub.stop();
        }
        #[cfg(feature = "console")]
        if let Some(control_plane) = self.control_plane {
            control_plane.stop();
        }

        let action = self.action.stop();
        let service = self.service.stop();
        let message = self.message.stop();

        #[cfg(all(feature = "ws", feature = "console"))]
        {
            return ws_handle.and(action).and(service).and(message);
        }
        #[cfg(all(feature = "ws", not(feature = "console")))]
        {
            return ws_handle.and(action).and(service).and(message);
        }
        #[cfg(all(feature = "console", not(feature = "ws")))]
        {
            return console.and(action).and(service).and(message);
        }
        #[cfg(not(any(feature = "ws", feature = "console")))]
        {
            action.and(service).and(message)
        }
    }
}

/// Ensure message/service/action share one broker id (for announce + federation hop-path).
fn normalize_broker_id(config: &mut RobotBusConfig) -> String {
    let id = [
        config.message.broker_id.as_str(),
        config.service.broker_id.as_str(),
        config.action.broker_id.as_str(),
    ]
    .into_iter()
    .find(|s| !s.is_empty())
    .map(str::to_string)
    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    config.message.broker_id = id.clone();
    config.service.broker_id = id.clone();
    config.action.broker_id = id.clone();
    id
}

fn transport_aliases(tcp: &str, channel: &str, discover: &DiscoverResponse) -> String {
    let mut parts = vec![tcp.to_string()];
    if let Some(dir) = discover.ipc_dir.as_deref().filter(|s| !s.is_empty()) {
        parts.push(ipc_endpoint_in(dir, channel));
    }
    if let Some(prefix) = discover
        .inproc_prefix
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        parts.push(inproc_endpoint_with_prefix(prefix, channel));
    }
    parts.join("  ")
}

/// Human-readable listen map printed after all buses (and the API port) are bound.
fn format_startup_banner(discover: &DiscoverResponse) -> String {
    let mut lines = vec![format!(
        "robot-bus {} ready  id={}",
        env!("CARGO_PKG_VERSION"),
        discover.broker_id
    )];
    let row = |label: &str, value: String| format!("  {label:<16}{value}");
    lines.push(String::new());
    #[cfg(feature = "ws")]
    {
        let ws_url = discover.api_url.trim_end_matches('/').to_string() + "/ws";
        lines.push(row("ws", ws_url));
    }
    if let Some(console) = discover.console_url.as_deref().filter(|s| !s.is_empty()) {
        lines.push(row("web console", console.to_string()));
    }
    lines.push(row(
        "message pub",
        transport_aliases(&discover.message_xsub, XSUB_CHANNEL, discover),
    ));
    lines.push(row(
        "message sub",
        transport_aliases(&discover.message_xpub, XPUB_CHANNEL, discover),
    ));
    lines.push(row(
        "service client",
        transport_aliases(
            &discover.service_frontend,
            SERVICE_FRONTEND_CHANNEL,
            discover,
        ),
    ));
    lines.push(row(
        "service worker",
        transport_aliases(
            &discover.service_backend,
            SERVICE_BACKEND_CHANNEL,
            discover,
        ),
    ));
    lines.push(row(
        "action client",
        transport_aliases(&discover.action_frontend, ACTION_FRONTEND_CHANNEL, discover),
    ));
    lines.push(row(
        "action worker",
        transport_aliases(&discover.action_backend, ACTION_BACKEND_CHANNEL, discover),
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_discover(ipc: bool) -> DiscoverResponse {
        DiscoverResponse {
            broker_id: "abc".into(),
            domain_id: 0,
            advertise_host: "127.0.0.1".into(),
            api_url: "http://127.0.0.1:15570".into(),
            message_xsub: "tcp://127.0.0.1:1".into(),
            message_xpub: "tcp://127.0.0.1:2".into(),
            service_frontend: "tcp://127.0.0.1:3".into(),
            service_backend: "tcp://127.0.0.1:4".into(),
            action_frontend: "tcp://127.0.0.1:5".into(),
            action_backend: "tcp://127.0.0.1:6".into(),
            ipc_dir: ipc.then(|| "/tmp/robot_bus/abc".into()),
            inproc_prefix: ipc.then(|| "robot_bus".into()),
            console_url: Some("http://127.0.0.1:15570".into()),
        }
    }

    #[test]
    fn startup_banner_lists_tcp_ipc_inproc() {
        let text = format_startup_banner(&sample_discover(true));
        assert!(text.contains("robot-bus"));
        assert!(text.contains("id=abc"));
        assert!(text.contains("tcp://127.0.0.1:1"));
        assert!(text.contains("ipc:///tmp/robot_bus/abc/message_bus_xsub.ipc"));
        assert!(text.contains("inproc://robot_bus/message_bus/xsub"));
        assert!(text.contains("tcp://127.0.0.1:5"));
        assert!(text.contains("inproc://robot_bus/action_bus/frontend"));
        let console_at = text.find("web console").expect("web console row");
        let message_at = text.find("message pub").expect("message row");
        assert!(console_at < message_at);
        assert!(text.contains("web console     http://127.0.0.1:15570"));
        #[cfg(feature = "ws")]
        {
            let ws_at = text.find("  ws ").expect("ws row");
            assert!(ws_at < console_at);
            assert!(text.contains("http://127.0.0.1:15570/ws"));
        }
        #[cfg(not(feature = "ws"))]
        {
            assert!(!text.contains("  ws "));
            assert!(!text.contains("/ws"));
        }
    }

    #[test]
    fn startup_banner_tcp_only_omits_ipc_inproc() {
        let text = format_startup_banner(&sample_discover(false));
        assert!(text.contains("tcp://127.0.0.1:1"));
        assert!(!text.contains("ipc://"));
        assert!(!text.contains("inproc://"));
    }
}
