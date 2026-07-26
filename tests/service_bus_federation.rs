//! Broker↔broker service-bus federation (static peers, hop-path anti-loop).

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::{ServiceBusConfig, ServicePeer};
#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "grpc")]
use robot_bus::GrpcBrokerConfig;
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{RobotBusBroker, RobotBusConfig};
use support::{free_port, lock_brokers};

fn connect_addr(bind: &str) -> String {
    bind.replace("tcp://0.0.0.0:", "tcp://127.0.0.1:")
        .replace("tcp://*:", "tcp://127.0.0.1:")
}

fn federated_service_config(
    broker_id: &str,
    peers: Vec<ServicePeer>,
    svc_fe: u16,
    svc_be: u16,
) -> RobotBusConfig {
    let mut ports = Vec::new();
    for _ in 0..6 {
        ports.push(free_port());
    }
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{}", ports[0]),
            xpub_bind: format!("tcp://127.0.0.1:{}", ports[1]),
            bind_all_transports: false,
            broker_id: broker_id.to_string(),
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{svc_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{svc_be}"),
            bind_all_transports: false,
            broker_id: broker_id.to_string(),
            peers,
            // Faster advertise / heartbeat for tests
            heartbeat_interval_ms: 200,
            heartbeat_timeout_ms: 2000,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", ports[2]),
            backend_bind: format!("tcp://127.0.0.1:{}", ports[3]),
            bind_all_transports: false,
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "grpc")]
        grpc: GrpcBrokerConfig {
            listen: format!("127.0.0.1:{}", ports[4])
                .parse()
                .expect("grpc listen"),
            cors_origins: Vec::new(),
        },
        #[cfg(feature = "console")]
        console: ConsoleBrokerConfig {
            enabled: false,
            listen: format!("127.0.0.1:{}", ports[5])
                .parse()
                .expect("console listen"),
        },
    }
}

fn alloc_svc_ports(n_brokers: usize) -> Vec<(u16, u16)> {
    let mut out = Vec::with_capacity(n_brokers);
    for _ in 0..n_brokers {
        out.push((free_port(), free_port()));
    }
    out
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
    let ports = alloc_svc_ports(2);
    let (a_fe, a_be) = ports[0];
    let (b_fe, b_be) = ports[1];

    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b_be)],
        a_fe,
        a_be,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a_be)],
        b_fe,
        b_be,
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

    let client_b = ServiceClient::new(Some(&connect_addr(&broker_b.service.frontend_bind)))
        .expect("client b");
    let reply = client_b
        .call("svc.a", b"ping", None, Some(Duration::from_secs(5)))
        .expect("call a from b");
    assert_eq!(reply, b"a:ping");

    let client_a = ServiceClient::new(Some(&connect_addr(&broker_a.service.frontend_bind)))
        .expect("client a");
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
    let ports = alloc_svc_ports(3);
    let (a_fe, a_be) = ports[0];
    let (b_fe, b_be) = ports[1];
    let (c_fe, c_be) = ports[2];

    // A — B — C (no direct A↔C)
    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b_be)],
        a_fe,
        a_be,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a_be), peer("broker-c", c_be)],
        b_fe,
        b_be,
    ))
    .expect("broker b");
    let broker_c = RobotBusBroker::start(federated_service_config(
        "broker-c",
        vec![peer("broker-b", b_be)],
        c_fe,
        c_be,
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

    let client_c = ServiceClient::new(Some(&connect_addr(&broker_c.service.frontend_bind)))
        .expect("client c");
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
    let ports = alloc_svc_ports(2);
    let (a_fe, a_be) = ports[0];
    let (b_fe, b_be) = ports[1];

    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b_be)],
        a_fe,
        a_be,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a_be)],
        b_fe,
        b_be,
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

    let client_b = ServiceClient::new(Some(&connect_addr(&broker_b.service.frontend_bind)))
        .expect("client b");
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
    let ports = alloc_svc_ports(2);
    let (a_fe, a_be) = ports[0];
    let (b_fe, b_be) = ports[1];

    let broker_a = RobotBusBroker::start(federated_service_config(
        "broker-a",
        vec![peer("broker-b", b_be)],
        a_fe,
        a_be,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_service_config(
        "broker-b",
        vec![peer("broker-a", a_be)],
        b_fe,
        b_be,
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

    let client_a = ServiceClient::new(Some(&connect_addr(&broker_a.service.frontend_bind)))
        .expect("client a");
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
