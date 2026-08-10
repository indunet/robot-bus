//! Broker↔broker action-bus federation (GoalTable-centric, hop-path anti-loop).

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "ws")]
use robot_bus::WsGatewayConfig;
use robot_bus::action_bus::{ActionClient, ActionKind};
use robot_bus::broker::action_bus::{ActionBusConfig, ActionPeer};
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::errors::BusError;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{DiscoveryConfig, RobotBusBroker, RobotBusConfig};
use support::{free_ports, lock_brokers};

fn connect_addr(bind: &str) -> String {
    bind.replace("tcp://0.0.0.0:", "tcp://127.0.0.1:")
        .replace("tcp://*:", "tcp://127.0.0.1:")
}

/// Per-broker ports: `(act_fe, act_be)` plus 6 binds for message/service/grpc/console.
/// Allocated in one `free_ports` batch so sequential probes cannot collide.
struct ActionBrokerPorts {
    act_fe: u16,
    act_be: u16,
    other: [u16; 6],
}

fn alloc_action_broker_ports(n: usize) -> Vec<ActionBrokerPorts> {
    let raw = free_ports(n * 8);
    raw.chunks(8)
        .map(|c| ActionBrokerPorts {
            act_fe: c[0],
            act_be: c[1],
            other: [c[2], c[3], c[4], c[5], c[6], c[7]],
        })
        .collect()
}

fn federated_action_config(
    broker_id: &str,
    peers: Vec<ActionPeer>,
    ports: &ActionBrokerPorts,
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
            frontend_bind: format!("tcp://127.0.0.1:{}", other[2]),
            backend_bind: format!("tcp://127.0.0.1:{}", other[3]),
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", ports.act_fe),
            backend_bind: format!("tcp://127.0.0.1:{}", ports.act_be),
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            peers,
            heartbeat_interval_ms: 200,
            heartbeat_timeout_ms: 1500,
            pending_timeout_ms: 3000,
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "ws")]
        grpc: WsGatewayConfig {
            listen: format!("127.0.0.1:{}", other[4])
                .parse()
                .expect("grpc listen"),
            cors_origins: Vec::new(),
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        #[cfg(feature = "console")]
        console: ConsoleBrokerConfig {
            enabled: false,
            listen: format!("127.0.0.1:{}", other[5])
                .parse()
                .expect("console listen"),
            cors_origins: vec![],
        },
    }
}

fn peer(broker_id: &str, backend: u16) -> ActionPeer {
    ActionPeer {
        backend: format!("tcp://127.0.0.1:{backend}"),
        broker_id: broker_id.to_string(),
    }
}

fn demo_handler(tag: &'static [u8]) -> Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> {
    Arc::new(move |body| {
        vec![
            ("FEEDBACK".into(), [b"fb:", tag].concat()),
            ("RESULT".into(), [b"done:", tag, b":", body].concat()),
        ]
    })
}

#[test]
fn two_brokers_bidirectional_actions() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_action_config(
        "broker-a",
        vec![peer("broker-b", b.act_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_action_config(
        "broker-b",
        vec![peer("broker-a", a.act_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_action(
        "act.a",
        demo_handler(b"a"),
        &connect_addr(&broker_a.action.backend_bind),
    )
    .expect("worker a");
    let worker_b = WorkerThread::spawn_action(
        "act.b",
        demo_handler(b"b"),
        &connect_addr(&broker_b.action.backend_bind),
    )
    .expect("worker b");

    thread::sleep(Duration::from_millis(600));

    let client_b =
        ActionClient::new(Some(&connect_addr(&broker_b.action.frontend_bind))).expect("client b");
    let msgs = client_b
        .send_goal("act.a", b"ping", None, Some(Duration::from_secs(5)))
        .expect("goal a from b");
    assert!(msgs.iter().any(|m| m.kind == ActionKind::Feedback));
    let result = msgs.iter().find(|m| m.kind == ActionKind::Result).unwrap();
    assert_eq!(result.body, b"done:a:ping");

    let client_a =
        ActionClient::new(Some(&connect_addr(&broker_a.action.frontend_bind))).expect("client a");
    let msgs = client_a
        .send_goal("act.b", b"pong", None, Some(Duration::from_secs(5)))
        .expect("goal b from a");
    let result = msgs.iter().find(|m| m.kind == ActionKind::Result).unwrap();
    assert_eq!(result.body, b"done:b:pong");

    worker_a.stop();
    worker_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn three_brokers_line_relay() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(3);
    let a = &ports[0];
    let b = &ports[1];
    let c = &ports[2];

    let broker_a = RobotBusBroker::start(federated_action_config(
        "broker-a",
        vec![peer("broker-b", b.act_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_action_config(
        "broker-b",
        vec![peer("broker-a", a.act_be), peer("broker-c", c.act_be)],
        b,
    ))
    .expect("broker b");
    let broker_c = RobotBusBroker::start(federated_action_config(
        "broker-c",
        vec![peer("broker-b", b.act_be)],
        c,
    ))
    .expect("broker c");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_action(
        "act.relay",
        demo_handler(b"rel"),
        &connect_addr(&broker_a.action.backend_bind),
    )
    .expect("worker a");

    thread::sleep(Duration::from_millis(900));

    let client_c =
        ActionClient::new(Some(&connect_addr(&broker_c.action.frontend_bind))).expect("client c");
    let msgs = client_c
        .send_goal("act.relay", b"hop", None, Some(Duration::from_secs(5)))
        .expect("relay goal");
    assert!(msgs.iter().any(|m| m.kind == ActionKind::Feedback));
    let result = msgs.iter().find(|m| m.kind == ActionKind::Result).unwrap();
    assert_eq!(result.body, b"done:rel:hop");

    worker_a.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
    broker_c.stop().expect("stop c");
}

#[test]
fn mesh_does_not_loop() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_action_config(
        "broker-a",
        vec![peer("broker-b", b.act_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_action_config(
        "broker-b",
        vec![peer("broker-a", a.act_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_action(
        "act.once",
        demo_handler(b"once"),
        &connect_addr(&broker_a.action.backend_bind),
    )
    .expect("worker a");

    thread::sleep(Duration::from_millis(600));

    let client_b =
        ActionClient::new(Some(&connect_addr(&broker_b.action.frontend_bind))).expect("client b");
    let msgs = client_b
        .send_goal("act.once", b"x", None, Some(Duration::from_secs(5)))
        .expect("goal");
    let results: Vec<_> = msgs
        .iter()
        .filter(|m| m.kind == ActionKind::Result)
        .collect();
    assert_eq!(results.len(), 1, "expected exactly one RESULT (no loop)");
    assert_eq!(results[0].body, b"done:once:x");

    worker_a.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn local_preferred_over_remote() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_action_config(
        "broker-a",
        vec![peer("broker-b", b.act_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_action_config(
        "broker-b",
        vec![peer("broker-a", a.act_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let worker_a = WorkerThread::spawn_action(
        "act.shared",
        Arc::new(|_| vec![("RESULT".into(), b"from-a".to_vec())]),
        &connect_addr(&broker_a.action.backend_bind),
    )
    .expect("worker a");
    let worker_b = WorkerThread::spawn_action(
        "act.shared",
        Arc::new(|_| vec![("RESULT".into(), b"from-b".to_vec())]),
        &connect_addr(&broker_b.action.backend_bind),
    )
    .expect("worker b");

    thread::sleep(Duration::from_millis(600));

    let client_a =
        ActionClient::new(Some(&connect_addr(&broker_a.action.frontend_bind))).expect("client a");
    for _ in 0..5 {
        let msgs = client_a
            .send_goal("act.shared", b"", None, Some(Duration::from_secs(3)))
            .expect("goal");
        let result = msgs.iter().find(|m| m.kind == ActionKind::Result).unwrap();
        assert_eq!(result.body, b"from-a");
    }

    worker_a.stop();
    worker_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn cancel_unknown_goal_on_federated_broker() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_action_config(
        "broker-a",
        vec![peer("broker-b", b.act_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_action_config(
        "broker-b",
        vec![peer("broker-a", a.act_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(200));

    let client_b =
        ActionClient::new(Some(&connect_addr(&broker_b.action.frontend_bind))).expect("client b");
    let err = client_b
        .cancel(
            "act.missing",
            "no-such-goal",
            b"",
            Some(Duration::from_secs(2)),
        )
        .expect_err("expected NO_GOAL");
    assert!(
        matches!(err, BusError::NoGoal { .. }),
        "unexpected error: {err:?}"
    );

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn peer_death_returns_worker_died() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_action_config(
        "broker-a",
        vec![peer("broker-b", b.act_be)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_action_config(
        "broker-b",
        vec![peer("broker-a", a.act_be)],
        b,
    ))
    .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let release = Arc::new(AtomicBool::new(false));
    let release_flag = release.clone();
    let worker_a = WorkerThread::spawn_action(
        "act.slow",
        Arc::new(move |_| {
            let deadline = Instant::now() + Duration::from_secs(30);
            while !release_flag.load(Ordering::Relaxed) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(50));
            }
            vec![("RESULT".into(), b"late".to_vec())]
        }),
        &connect_addr(&broker_a.action.backend_bind),
    )
    .expect("worker a");

    thread::sleep(Duration::from_millis(600));

    let client_b =
        ActionClient::new(Some(&connect_addr(&broker_b.action.frontend_bind))).expect("client b");
    let gid = client_b
        .submit_goal("act.slow", b"x", None)
        .expect("submit");

    thread::sleep(Duration::from_millis(200));
    // Kill the hosting broker while the goal is in flight; then unblock the worker thread.
    broker_a.stop().expect("stop a");
    release.store(true, Ordering::Relaxed);
    drop(worker_a);

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut saw_died = false;
    while Instant::now() < deadline {
        match client_b.recv_message(Some(Duration::from_millis(500))) {
            Ok(msg) if msg.goal_id == gid && msg.kind == ActionKind::Result => {
                if matches!(
                    robot_bus::parse_error_body(&msg.body),
                    Some(BusError::WorkerDied { .. })
                ) {
                    saw_died = true;
                    break;
                }
            }
            Ok(_) => continue,
            Err(BusError::Timeout(_)) => continue,
            Err(_) => break,
        }
    }
    assert!(saw_died, "expected WORKER_DIED after peer broker stopped");

    broker_b.stop().expect("stop b");
}
