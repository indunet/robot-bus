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
use support::{free_port, lock_brokers};

fn connect_addr(bind: &str) -> String {
    bind.replace("tcp://0.0.0.0:", "tcp://127.0.0.1:")
        .replace("tcp://*:", "tcp://127.0.0.1:")
}

/// Ephemeral TCP binds for one broker; message XSUB/XPUB need not be adjacent
/// because tests set [`MessagePeer`] explicitly.
fn federated_bus_config(
    broker_id: &str,
    peers: Vec<MessagePeer>,
    msg_xsub: u16,
    msg_xpub: u16,
) -> RobotBusConfig {
    let mut ports = Vec::new();
    for _ in 0..6 {
        ports.push(free_port());
    }
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{msg_xsub}"),
            xpub_bind: format!("tcp://127.0.0.1:{msg_xpub}"),
            snd_hwm: 100,
            rcv_hwm: 100,
            bind_all_transports: false,
            broker_id: broker_id.to_string(),
            peers,
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", ports[0]),
            backend_bind: format!("tcp://127.0.0.1:{}", ports[1]),
            bind_all_transports: false,
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
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
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

fn alloc_msg_ports(n_brokers: usize) -> Vec<(u16, u16)> {
    let mut out = Vec::with_capacity(n_brokers);
    for _ in 0..n_brokers {
        out.push((free_port(), free_port()));
    }
    out
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
    let ports = alloc_msg_ports(2);
    let (a_xsub, a_xpub) = ports[0];
    let (b_xsub, b_xpub) = ports[1];

    let peer_a = MessagePeer {
        xpub: format!("tcp://127.0.0.1:{a_xpub}"),
        xsub: format!("tcp://127.0.0.1:{a_xsub}"),
    };
    let peer_b = MessagePeer {
        xpub: format!("tcp://127.0.0.1:{b_xpub}"),
        xsub: format!("tcp://127.0.0.1:{b_xsub}"),
    };

    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![peer_b.clone()],
        a_xsub,
        a_xpub,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![peer_a],
        b_xsub,
        b_xpub,
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
    let ports = alloc_msg_ports(3);
    let (a_xsub, a_xpub) = ports[0];
    let (b_xsub, b_xpub) = ports[1];
    let (c_xsub, c_xpub) = ports[2];

    let peer = |xsub: u16, xpub: u16| MessagePeer {
        xpub: format!("tcp://127.0.0.1:{xpub}"),
        xsub: format!("tcp://127.0.0.1:{xsub}"),
    };

    // A — B — C (no direct A↔C)
    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![peer(b_xsub, b_xpub)],
        a_xsub,
        a_xpub,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![peer(a_xsub, a_xpub), peer(c_xsub, c_xpub)],
        b_xsub,
        b_xpub,
    ))
    .expect("broker b");
    let broker_c = RobotBusBroker::start(federated_bus_config(
        "broker-c",
        vec![peer(b_xsub, b_xpub)],
        c_xsub,
        c_xpub,
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
    let ports = alloc_msg_ports(2);
    let (a_xsub, a_xpub) = ports[0];
    let (b_xsub, b_xpub) = ports[1];

    let peer_a = MessagePeer {
        xpub: format!("tcp://127.0.0.1:{a_xpub}"),
        xsub: format!("tcp://127.0.0.1:{a_xsub}"),
    };
    let peer_b = MessagePeer {
        xpub: format!("tcp://127.0.0.1:{b_xpub}"),
        xsub: format!("tcp://127.0.0.1:{b_xsub}"),
    };

    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![peer_b],
        a_xsub,
        a_xpub,
    ))
    .expect("broker a");
    let broker_b = RobotBusBroker::start(federated_bus_config(
        "broker-b",
        vec![peer_a],
        b_xsub,
        b_xpub,
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
