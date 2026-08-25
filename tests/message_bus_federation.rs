//! Broker↔broker message-bus federation (static peers, hop-path anti-loop).

mod support;

use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "ws")]
use robot_bus::WsGatewayConfig;
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::{BusConfig, MessagePeer};
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::{DiscoveryConfig, Publisher, RobotBusBroker, RobotBusConfig, Subscriber};
use support::{free_ports, lock_brokers};

fn connect_addr(bind: &str) -> String {
    bind.replace("tcp://0.0.0.0:", "tcp://127.0.0.1:")
        .replace("tcp://*:", "tcp://127.0.0.1:")
}

/// Per-broker ports: `(xsub, xpub)` plus 6 binds for service/action/grpc/console.
struct MessageBrokerPorts {
    xsub: u16,
    xpub: u16,
    other: [u16; 6],
}

fn alloc_message_broker_ports(n: usize) -> Vec<MessageBrokerPorts> {
    let raw = free_ports(n * 8);
    raw.chunks(8)
        .map(|c| MessageBrokerPorts {
            xsub: c[0],
            xpub: c[1],
            other: [c[2], c[3], c[4], c[5], c[6], c[7]],
        })
        .collect()
}

/// Ephemeral TCP binds for one broker; message XSUB/XPUB need not be adjacent
/// because tests set [`MessagePeer`] explicitly.
fn federated_bus_config(
    broker_id: &str,
    peers: Vec<MessagePeer>,
    ports: &MessageBrokerPorts,
) -> RobotBusConfig {
    let other = &ports.other;
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{}", ports.xsub),
            xpub_bind: format!("tcp://127.0.0.1:{}", ports.xpub),
            snd_hwm: 100,
            rcv_hwm: 100,
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            peers,
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", other[0]),
            backend_bind: format!("tcp://127.0.0.1:{}", other[1]),
            bind_all_transports: false,
            bind_opts: Default::default(),
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
            docs_enabled: true,
            listen: format!("127.0.0.1:{}", other[5])
                .parse()
                .expect("console listen"),
            cors_origins: vec![],
        },
    }
}

fn msg_peer(ports: &MessageBrokerPorts) -> MessagePeer {
    MessagePeer {
        xpub: format!("tcp://127.0.0.1:{}", ports.xpub),
        xsub: format!("tcp://127.0.0.1:{}", ports.xsub),
    }
}

fn recv_exact(sub: &Subscriber, expect: &[u8], timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        match sub.receive(Some(left.min(Duration::from_millis(50)))) {
            Ok((topic, payload)) if payload == expect => return topic,
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
    panic!("timed out waiting for payload {:?}", expect);
}

fn count_matching(sub: &Subscriber, expect: &[u8], window: Duration) -> usize {
    let deadline = Instant::now() + window;
    let mut n = 0;
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        match sub.receive(Some(left.min(Duration::from_millis(20)))) {
            Ok((_, payload)) if payload == expect => n += 1,
            _ => {}
        }
    }
    n
}

#[test]
fn two_brokers_bidirectional_topics() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_bus_config("broker-a", vec![msg_peer(b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config("broker-b", vec![msg_peer(a)], b))
        .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let sub_b = Subscriber::new(Some(&connect_addr(&broker_b.message.xpub_bind))).expect("sub b");
    sub_b.subscribe("fleet/pose").expect("subscribe");
    let sub_a = Subscriber::new(Some(&connect_addr(&broker_a.message.xpub_bind))).expect("sub a");
    sub_a.subscribe("fleet/pose").expect("subscribe");

    // Demand must reach the publisher's broker via peer SUB → XPUB.
    thread::sleep(Duration::from_millis(300));

    let pub_a = Publisher::new(Some(&connect_addr(&broker_a.message.xsub_bind))).expect("pub a");
    let pub_b = Publisher::new(Some(&connect_addr(&broker_b.message.xsub_bind))).expect("pub b");
    thread::sleep(Duration::from_millis(100));

    pub_a.publish("fleet/pose", b"from-a").expect("publish a");
    assert_eq!(
        recv_exact(&sub_b, b"from-a", Duration::from_secs(2)),
        "fleet/pose"
    );

    pub_b.publish("fleet/pose", b"from-b").expect("publish b");
    assert_eq!(
        recv_exact(&sub_a, b"from-b", Duration::from_secs(2)),
        "fleet/pose"
    );

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn three_brokers_line_relay() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(3);
    let a = &ports[0];
    let b = &ports[1];
    let c = &ports[2];

    // A — B — C (no direct A↔C)
    let broker_a = RobotBusBroker::start(federated_bus_config("broker-a", vec![msg_peer(b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![msg_peer(a), msg_peer(c)],
        b,
    ))
    .expect("broker b");
    let broker_c = RobotBusBroker::start(federated_bus_config("broker-c", vec![msg_peer(b)], c))
        .expect("broker c");

    thread::sleep(Duration::from_millis(100));

    let sub_c = Subscriber::new(Some(&connect_addr(&broker_c.message.xpub_bind))).expect("sub c");
    sub_c.subscribe("relay/topic").expect("subscribe");
    thread::sleep(Duration::from_millis(400));

    let pub_a = Publisher::new(Some(&connect_addr(&broker_a.message.xsub_bind))).expect("pub a");
    thread::sleep(Duration::from_millis(100));
    pub_a.publish("relay/topic", b"hop").expect("publish");

    assert_eq!(
        recv_exact(&sub_c, b"hop", Duration::from_secs(3)),
        "relay/topic"
    );

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
    broker_c.stop().expect("stop c");
}

#[test]
fn mesh_does_not_storm() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_bus_config("broker-a", vec![msg_peer(b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config("broker-b", vec![msg_peer(a)], b))
        .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let sub_b = Subscriber::new(Some(&connect_addr(&broker_b.message.xpub_bind))).expect("sub b");
    sub_b.subscribe("once").expect("subscribe");
    thread::sleep(Duration::from_millis(300));

    let pub_a = Publisher::new(Some(&connect_addr(&broker_a.message.xsub_bind))).expect("pub a");
    thread::sleep(Duration::from_millis(100));
    pub_a.publish("once", b"unique").expect("publish");

    let n = count_matching(&sub_b, b"unique", Duration::from_millis(500));
    assert_eq!(n, 1, "expected exactly one delivery, got {n} (loop?)");

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn reserved_robot_bus_topics_stay_local() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    let broker_a = RobotBusBroker::start(federated_bus_config("broker-a", vec![msg_peer(b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config("broker-b", vec![msg_peer(a)], b))
        .expect("broker b");

    thread::sleep(Duration::from_millis(100));

    let sub_b = Subscriber::new(Some(&connect_addr(&broker_b.message.xpub_bind))).expect("sub b");
    sub_b
        .subscribe(robot_bus::console_topics::STATUS)
        .expect("subscribe status");
    sub_b.subscribe("fleet/pose").expect("subscribe pose");

    let sub_a = Subscriber::new(Some(&connect_addr(&broker_a.message.xpub_bind))).expect("sub a");
    sub_a
        .subscribe(robot_bus::console_topics::STATUS)
        .expect("subscribe local status");

    // Demand must reach peers for the federated user topic.
    thread::sleep(Duration::from_millis(300));

    let pub_a = Publisher::new(Some(&connect_addr(&broker_a.message.xsub_bind))).expect("pub a");
    thread::sleep(Duration::from_millis(100));

    pub_a
        .publish(robot_bus::console_topics::STATUS, b"status-a")
        .expect("publish status");

    assert_eq!(
        recv_exact(&sub_a, b"status-a", Duration::from_secs(2)),
        robot_bus::console_topics::STATUS,
        "local console status must still deliver"
    );
    let leaked = count_matching(&sub_b, b"status-a", Duration::from_millis(500));
    assert_eq!(
        leaked, 0,
        "reserved /robot_bus/status must not cross federation"
    );

    pub_a.publish("fleet/pose", b"from-a").expect("publish pose");
    assert_eq!(
        recv_exact(&sub_b, b"from-a", Duration::from_secs(2)),
        "fleet/pose",
        "user topics must still federate"
    );

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

/// HTTP discover needs the gRPC gateway listener (`GET /api/v1/discover`).
/// Skip under `cargo test --no-default-features` where no API server is started.
#[cfg(feature = "ws")]
#[test]
fn federation_peer_via_api_discover() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let a = &ports[0];
    let b = &ports[1];

    // Bring up both APIs (no federation yet) so each side can be discovered.
    let mut cfg_a = federated_bus_config("api-peer-a", Vec::new(), a);
    let mut cfg_b = federated_bus_config("api-peer-b", Vec::new(), b);
    cfg_a.discovery.advertise_host = Some("127.0.0.1".into());
    cfg_b.discovery.advertise_host = Some("127.0.0.1".into());
    #[cfg(feature = "console")]
    {
        cfg_a.console.enabled = false;
        cfg_b.console.enabled = false;
    }
    let broker_a = RobotBusBroker::start(cfg_a).expect("broker a");
    let broker_b = RobotBusBroker::start(cfg_b).expect("broker b");
    thread::sleep(Duration::from_millis(150));

    let api_a = format!("127.0.0.1:{}", a.other[4]);
    let api_b = format!("127.0.0.1:{}", b.other[4]);
    let peer_a = robot_bus::resolve_peer_from_api(&api_a).expect("discover a");
    let peer_b = robot_bus::resolve_peer_from_api(&api_b).expect("discover b");
    assert_eq!(peer_a.broker_id, "api-peer-a");
    assert_eq!(peer_b.broker_id, "api-peer-b");
    assert_eq!(peer_a.message.xpub, format!("tcp://127.0.0.1:{}", a.xpub));
    assert_eq!(peer_a.message.xsub, format!("tcp://127.0.0.1:{}", a.xsub));

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
    thread::sleep(Duration::from_millis(100));

    // Restart with bidirectional peers resolved from /api/v1/discover (not XPUB-1).
    let broker_a = RobotBusBroker::start(federated_bus_config(
        "api-peer-a",
        vec![peer_b.message],
        a,
    ))
    .expect("broker a federated");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "api-peer-b",
        vec![peer_a.message],
        b,
    ))
    .expect("broker b federated");
    thread::sleep(Duration::from_millis(200));

    let sub_b = Subscriber::new(Some(&connect_addr(&broker_b.message.xpub_bind))).expect("sub b");
    sub_b.subscribe("fleet/api_peer").expect("subscribe");
    thread::sleep(Duration::from_millis(300));

    let pub_a = Publisher::new(Some(&connect_addr(&broker_a.message.xsub_bind))).expect("pub a");
    thread::sleep(Duration::from_millis(100));
    pub_a
        .publish("fleet/api_peer", b"via-api")
        .expect("publish");
    assert_eq!(
        recv_exact(&sub_b, b"via-api", Duration::from_secs(2)),
        "fleet/api_peer"
    );

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}
