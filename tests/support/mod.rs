//! Test helpers: inline message proxy and in-process robot bus broker.

#![allow(dead_code)] // shared across integration tests; not every helper is used in every crate

use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "grpc")]
use robot_bus::GrpcBrokerConfig;
use robot_bus::{RobotBusBroker, RobotBusConfig};
use zmq::{Context, Socket, SocketType};

static PROXY_ID: AtomicU64 = AtomicU64::new(0);

/// `bind_all` uses fixed inproc/ipc names — only one full broker at a time.
static BROKER_LOCK: Mutex<()> = Mutex::new(());

pub fn lock_brokers() -> MutexGuard<'static, ()> {
    BROKER_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

pub fn ephemeral_robot_bus_config() -> RobotBusConfig {
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{}", free_port()),
            xpub_bind: format!("tcp://127.0.0.1:{}", free_port()),
            bind_all_transports: false,
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
            backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
            bind_all_transports: false,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
            backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
            bind_all_transports: false,
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "grpc")]
        grpc: GrpcBrokerConfig {
            listen: format!("127.0.0.1:{}", free_port())
                .parse()
                .expect("grpc listen"),
            cors_origins: Vec::new(),
        },
        #[cfg(feature = "console")]
        console: ConsoleBrokerConfig {
            enabled: false,
            listen: format!("127.0.0.1:{}", free_port())
                .parse()
                .expect("console listen"),
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
        let xsub_port = free_port();
        let xpub_port = free_port();
        let xsub_endpoint = format!("tcp://127.0.0.1:{xsub_port}");
        let xpub_endpoint = format!("tcp://127.0.0.1:{xpub_port}");
        xsub.bind(&xsub_endpoint).expect("bind xsub");
        xpub.bind(&xpub_endpoint).expect("bind xpub");
        xsub.set_linger(0).expect("linger");
        xpub.set_linger(0).expect("linger");

        let id = PROXY_ID.fetch_add(1, Ordering::Relaxed);
        let control_name = format!("inproc://message-proxy-ctl-{id}");
        control.bind(&control_name).expect("bind control");
        control_client.connect(&control_name).expect("connect control");

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
    _guard: MutexGuard<'static, ()>,
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
