//! Test helpers: inline message proxy and in-process robot bus broker.

#![allow(dead_code)] // shared across integration tests; not every helper is used in every crate

use std::fs::OpenOptions;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "ws")]
use robot_bus::WsGatewayConfig;
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::{DiscoveryConfig, RobotBusBroker, RobotBusConfig};
use zmq::{Context, Socket, SocketType};

static PROXY_ID: AtomicU64 = AtomicU64::new(0);

/// Cross-process lock so parallel `cargo test` binaries cannot race on
/// `free_ports` → bind TOCTOU (in-process `Mutex` is per test binary only).
pub struct BrokerLockGuard {
    _file: std::fs::File,
}

/// Serialize broker integration tests across threads **and** test binaries.
///
/// `cargo test` runs many `tests/*.rs` targets in parallel; each gets its own
/// copy of this module, so a process-local `Mutex` does not help. We `flock`
/// a temp file instead (released when the process exits, even on panic).
pub fn lock_brokers() -> BrokerLockGuard {
    let path = broker_lock_path();
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("open broker lock {}: {e}", path.display()));
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let rc = unsafe { libc::flock(fd, libc::LOCK_EX) };
        if rc != 0 {
            panic!(
                "flock {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            );
        }
    }
    BrokerLockGuard { _file: file }
}

fn broker_lock_path() -> PathBuf {
    std::env::temp_dir().join("robot-bus-integration-broker.lock")
}

/// Probe an unused TCP port. **TOCTOU**: the port is released when this returns,
/// so a later bind can still fail under contention. Prefer binding `…:0` and
/// reading [`zmq::Socket::get_last_endpoint`] when the bind is under our control.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

fn last_endpoint(sock: &Socket) -> String {
    match sock.get_last_endpoint().expect("last_endpoint") {
        Ok(s) => s,
        Err(_) => panic!("endpoint not utf8"),
    }
}

/// Distinct ephemeral TCP ports for one config snapshot.
///
/// Hold all listeners together so this batch never picks the same port twice
/// (sequential [`free_port`] can return duplicates and flake on the second bind).
/// Still subject to TOCTOU vs other processes after the listeners are dropped;
/// broker tests serialize with [`lock_brokers`]. Prefer ZMQ `…:0` bind when possible.
pub fn free_ports(n: usize) -> Vec<u16> {
    let listeners: Vec<TcpListener> = (0..n)
        .map(|_| TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port"))
        .collect();
    listeners
        .iter()
        .map(|l| l.local_addr().expect("local addr").port())
        .collect()
    // listeners dropped here — ports free for the subsequent ZMQ/TCP binds
}

pub fn ephemeral_robot_bus_config() -> RobotBusConfig {
    // Bind `:0` and read the real endpoints from `RobotBusBroker` after start.
    // `free_ports()` is TOCTOU: parallel tests (MessageProxy, other cargo-test
    // binaries) can steal the probed port between drop and ZMQ/HTTP bind, which
    // surfaces as "message bus failed to report bound endpoints".
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: "tcp://127.0.0.1:0".into(),
            xpub_bind: "tcp://127.0.0.1:0".into(),
            bind_all_transports: false,
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: "tcp://127.0.0.1:0".into(),
            backend_bind: "tcp://127.0.0.1:0".into(),
            bind_all_transports: false,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: "tcp://127.0.0.1:0".into(),
            backend_bind: "tcp://127.0.0.1:0".into(),
            bind_all_transports: false,
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "ws")]
        ws: WsGatewayConfig {
            listen: "127.0.0.1:0".parse().expect("grpc listen"),
            cors_origins: Vec::new(),
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        #[cfg(feature = "console")]
        console: ConsoleBrokerConfig {
            enabled: false,
            tank_enabled: false,
            listen: "127.0.0.1:0".parse().expect("console listen"),
            cors_origins: vec![],
        },
    }
}

pub struct MessageProxy {
    pub xsub_endpoint: String,
    pub xpub_endpoint: String,
    handle: Option<JoinHandle<()>>,
    control_client: Socket,
    #[allow(dead_code)]
    context: Context,
}

impl MessageProxy {
    pub fn spawn() -> Self {
        let context = Context::new();
        let mut xsub = context.socket(SocketType::XSUB).expect("xsub");
        let mut xpub = context.socket(SocketType::XPUB).expect("xpub");
        let mut control = context.socket(SocketType::PAIR).expect("control");
        let control_client = context.socket(SocketType::PAIR).expect("control client");
        // Bind `:0` then read the real endpoint — avoids free_port() TOCTOU where
        // another test (or the sibling xpub bind) can steal the probed port.
        xsub.bind("tcp://127.0.0.1:0").expect("bind xsub");
        xpub.bind("tcp://127.0.0.1:0").expect("bind xpub");
        let xsub_endpoint = last_endpoint(&xsub);
        let xpub_endpoint = last_endpoint(&xpub);
        xsub.set_linger(0).expect("linger");
        xpub.set_linger(0).expect("linger");

        let id = PROXY_ID.fetch_add(1, Ordering::Relaxed);
        let control_name = format!("inproc://message-proxy-ctl-{id}");
        control.bind(&control_name).expect("bind control");
        control_client
            .connect(&control_name)
            .expect("connect control");

        let handle = thread::spawn(move || {
            let _ = zmq::proxy_steerable(&mut xsub, &mut xpub, &mut control);
        });
        thread::sleep(Duration::from_millis(50));
        Self {
            xsub_endpoint,
            xpub_endpoint,
            handle: Some(handle),
            control_client,
            context,
        }
    }
}

impl Drop for MessageProxy {
    fn drop(&mut self) {
        let _ = self.control_client.send(b"TERMINATE" as &[u8], 0);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// In-process robot bus (all buses, and gRPC when the `grpc` feature is on)
/// with ephemeral TCP ports.
///
/// `frontend_endpoint` / `backend_endpoint` are the service or action pair
/// selected by [`Self::spawn_service`] / [`Self::spawn_action`].
pub struct BrokerProcess {
    pub frontend_endpoint: String,
    pub backend_endpoint: String,
    _guard: BrokerLockGuard,
    _broker: RobotBusBroker,
}

impl BrokerProcess {
    fn spawn_with(config: RobotBusConfig, service_endpoints: bool) -> Self {
        let guard = lock_brokers();
        let broker = RobotBusBroker::start(config).expect("start RobotBusBroker");
        let (frontend_endpoint, backend_endpoint) = if service_endpoints {
            (
                broker.service.frontend_bind.clone(),
                broker.service.backend_bind.clone(),
            )
        } else {
            (
                broker.action.frontend_bind.clone(),
                broker.action.backend_bind.clone(),
            )
        };
        Self {
            frontend_endpoint,
            backend_endpoint,
            _guard: guard,
            _broker: broker,
        }
    }

    pub fn spawn_service() -> Self {
        let mut config = ephemeral_robot_bus_config();
        config.service.heartbeat_interval_ms = 200;
        config.service.heartbeat_timeout_ms = 600;
        Self::spawn_with(config, true)
    }

    pub fn spawn_action() -> Self {
        let mut config = ephemeral_robot_bus_config();
        config.action.heartbeat_interval_ms = 100;
        config.action.heartbeat_timeout_ms = 600;
        config.action.pending_timeout_ms = 200;
        Self::spawn_with(config, false)
    }
}
