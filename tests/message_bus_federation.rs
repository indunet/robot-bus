//! Broker↔broker message-bus federation (static peers, hop-path anti-loop).

mod support;

use std::thread;
use std::time::{Duration, Instant};

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::{BusConfig, MessagePeer};
use robot_bus::broker::service_bus::ServiceBusConfig;
#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
#[cfg(feature = "grpc")]
use robot_bus::GrpcBrokerConfig;
use robot_bus::{
    DiscoveryConfig, Publisher, RobotBusBroker, RobotBusConfig, Subscriber,
};
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
            broker_id: broker_id.to_string(),
            peers,
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", other[0]),
            backend_bind: format!("tcp://127.0.0.1:{}", other[1]),
            bind_all_transports: false,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", other[2]),
            backend_bind: format!("tcp://127.0.0.1:{}", other[3]),
            bind_all_transports: false,
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "grpc")]
        grpc: GrpcBrokerConfig {
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

    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![msg_peer(b)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![msg_peer(a)],
        b,
    ))
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
    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![msg_peer(b)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![msg_peer(a), msg_peer(c)],
        b,
    ))
    .expect("broker b");
    let broker_c = RobotBusBroker::start(federated_bus_config(
        "broker-c",
        vec![msg_peer(b)],
        c,
    ))
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

    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![msg_peer(b)],
        a,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![msg_peer(a)],
        b,
    ))
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
