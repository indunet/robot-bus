//! Broker↔broker service-bus federation (static peers, hop-path anti-loop).

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "ws")]
use robot_bus::WsGatewayConfig;
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::{ServiceBusConfig, ServicePeer};
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{DiscoveryConfig, RobotBusBroker, RobotBusConfig};
use support::{free_ports, lock_brokers};

fn connect_addr(bind: &str) -> String {
    bind.replace("tcp://0.0.0.0:", "tcp://127.0.0.1:")
        .replace("tcp://*:", "tcp://127.0.0.1:")
}

/// Per-broker ports: `(svc_fe, svc_be)` plus 6 binds for message/action/grpc/console.
struct ServiceBrokerPorts {
    svc_fe: u16,
    svc_be: u16,
    other: [u16; 6],
}

fn alloc_service_broker_ports(n: usize) -> Vec<ServiceBrokerPorts> {
    let raw = free_ports(n * 8);
    raw.chunks(8)
        .map(|c| ServiceBrokerPorts {
            svc_fe: c[0],
            svc_be: c[1],
            other: [c[2], c[3], c[4], c[5], c[6], c[7]],
        })
        .collect()
}

fn federated_service_config(
    broker_id: &str,
    peers: Vec<ServicePeer>,
    ports: &ServiceBrokerPorts,
) -> RobotBusConfig {
    let other = &ports.other;
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{}", other[0]),
            xpub_bind: format!("tcp://127.0.0.1:{}", other[1]),
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", ports.svc_fe),
            backend_bind: format!("tcp://127.0.0.1:{}", ports.svc_be),
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            peers,
            // Faster advertise / heartbeat for tests
            heartbeat_interval_ms: 200,
            heartbeat_timeout_ms: 2000,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", other[2]),
            backend_bind: format!("tcp://127.0.0.1:{}", other[3]),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "ws")]
        ws: WsGatewayConfig {
            listen: format!("127.0.0.1:{}", other[4])
                .parse()
                .expect("ws listen"),
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
            listen: format!("127.0.0.1:{}", other[5])
                .parse()
                .expect("console listen"),
            cors_origins: vec![],
        },
    }
}

fn peer(broker_id: &str, backend: u16) -> ServicePeer {
    ServicePeer {
        backend: format!("tcp://127.0.0.1:{backend}"),
        broker_id: broker_id.to_string(),
    }
}

#[test]
fn two_brokers_bidirectional_services() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b.svc_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a.svc_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_service(
        "svc.a",
        Arc::new(|body| [b"a:", body].concat()),
        &connect_addr(&broker_a.service.backend_bind),
    )
    .expect("worker a");
    let worker_b = WorkerThread::spawn_service(
        "svc.b",
        Arc::new(|body| [b"b:", body].concat()),
        &connect_addr(&broker_b.service.backend_bind),
    )
    .expect("worker b");

    // Let READY_FED propagate both ways.
    thread::sleep(Duration::from_millis(500));

    let client_b =
        ServiceClient::new(Some(&connect_addr(&broker_b.service.frontend_bind))).expect("client b");
    let reply = client_b
        .call("svc.a", b"ping", None, Some(Duration::from_secs(5)))
        .expect("call a from b");
    assert_eq!(reply, b"a:ping");

    let client_a =
        ServiceClient::new(Some(&connect_addr(&broker_a.service.frontend_bind))).expect("client a");
    let reply = client_a
        .call("svc.b", b"pong", None, Some(Duration::from_secs(5)))
        .expect("call b from a");
    assert_eq!(reply, b"b:pong");

    worker_a.stop();
    worker_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn three_brokers_line_relay() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(3);
    let a = &ports[0];
    let b = &ports[1];
    let c = &ports[2];

    // A — B — C (no direct A↔C)
    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b.svc_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a.svc_be), peer("broker-c", c.svc_be)],
        b,
    ))
    .expect("broker b");
    let broker_c = RobotBusBroker::start(federated_service_config(
        "broker-c",
        vec![peer("broker-b", b.svc_be)],
        c,
    ))
    .expect("broker c");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_service(
        "svc.relay",
        Arc::new(|body| [b"relay:", body].concat()),
        &connect_addr(&broker_a.service.backend_bind),
    )
    .expect("worker a");

    // Demand walks A→B→C via READY_FED re-advertisement.
    thread::sleep(Duration::from_millis(800));

    let client_c =
        ServiceClient::new(Some(&connect_addr(&broker_c.service.frontend_bind))).expect("client c");
    let reply = client_c
        .call("svc.relay", b"hop", None, Some(Duration::from_secs(5)))
        .expect("call via B");
    assert_eq!(reply, b"relay:hop");

    worker_a.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
    broker_c.stop().expect("stop c");
}

#[test]
fn mesh_does_not_loop() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b.svc_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a.svc_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_service(
        "svc.once",
        Arc::new(|body| [b"once:", body].concat()),
        &connect_addr(&broker_a.service.backend_bind),
    )
    .expect("worker a");

    thread::sleep(Duration::from_millis(500));

    let client_b =
        ServiceClient::new(Some(&connect_addr(&broker_b.service.frontend_bind))).expect("client b");
    let reply = client_b
        .call("svc.once", b"x", None, Some(Duration::from_secs(5)))
        .expect("call once");
    assert_eq!(reply, b"once:x");

    // Second call still succeeds exactly once (no storm / hang from A↔B loop).
    let reply2 = client_b
        .call("svc.once", b"y", None, Some(Duration::from_secs(5)))
        .expect("call once again");
    assert_eq!(reply2, b"once:y");

    worker_a.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn local_preferred_over_remote() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b.svc_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a.svc_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_service(
        "svc.shared",
        Arc::new(|_| b"from-a".to_vec()),
        &connect_addr(&broker_a.service.backend_bind),
    )
    .expect("worker a");
    let worker_b = WorkerThread::spawn_service(
        "svc.shared",
        Arc::new(|_| b"from-b".to_vec()),
        &connect_addr(&broker_b.service.backend_bind),
    )
    .expect("worker b");

    thread::sleep(Duration::from_millis(500));

    let client_a =
        ServiceClient::new(Some(&connect_addr(&broker_a.service.frontend_bind))).expect("client a");
    for _ in 0..5 {
        let reply = client_a
            .call("svc.shared", b"", None, Some(Duration::from_secs(3)))
            .expect("call");
        assert_eq!(reply, b"from-a", "A client must hit local worker only");
    }

    worker_a.stop();
    worker_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn peer_death_inflight_returns_worker_died() {
    use robot_bus::errors::BusError;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Instant;

    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let mut cfg_a = federated_service_config("broker-a", vec![peer("broker-b", b.svc_be)], a);
    cfg_a.service.heartbeat_interval_ms = 100;
    cfg_a.service.heartbeat_timeout_ms = 400;
    let mut cfg_b = federated_service_config("broker-b", vec![peer("broker-a", a.svc_be)], b);
    cfg_b.service.heartbeat_interval_ms = 100;
    cfg_b.service.heartbeat_timeout_ms = 400;

    let broker_a = RobotBusBroker::start(cfg_a).expect("broker a");
    let broker_b = RobotBusBroker::start(cfg_b).expect("broker b");
    thread::sleep(Duration::from_millis(100));

    let release = Arc::new(AtomicBool::new(false));
    let release_flag = release.clone();
    let worker_a = WorkerThread::spawn_service(
        "svc.slow",
        Arc::new(move |_| {
            let deadline = Instant::now() + Duration::from_secs(30);
            while !release_flag.load(Ordering::Relaxed) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(50));
            }
            b"late".to_vec()
        }),
        &connect_addr(&broker_a.service.backend_bind),
    )
    .expect("worker a");

    thread::sleep(Duration::from_millis(600));

    let client_b =
        ServiceClient::new(Some(&connect_addr(&broker_b.service.frontend_bind))).expect("client b");

    let handle = thread::spawn(move || {
        client_b.call("svc.slow", b"x", Some("r1"), Some(Duration::from_secs(8)))
    });

    thread::sleep(Duration::from_millis(200));
    broker_a.stop().expect("stop a");
    release.store(true, Ordering::Relaxed);
    drop(worker_a);

    let result = handle.join().expect("join call");
    match result {
        Err(BusError::WorkerDied { .. }) | Err(BusError::NoWorker { .. }) => {}
        other => panic!("expected WorkerDied/NoWorker after peer death, got {other:?}"),
    }

    broker_b.stop().expect("stop b");
}
