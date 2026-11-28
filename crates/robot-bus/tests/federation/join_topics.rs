//! One-sided federation: only the joining broker lists `--peer`; the joined broker is
//! started without any peer configuration and never restarted.

use robot_bus::zmq_helpers as zmq;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::lock_brokers;
use crate::message_bus_federation::{
    alloc_message_broker_ports, connect_addr, count_matching, federated_bus_config, join_peer,
    msg_peer, recv_exact,
};
use robot_bus::{Node, NodeOptions, Publisher, QosProfile, RobotBusBroker, Subscriber};

fn subscriber(broker: &RobotBusBroker, topic: &str) -> Subscriber {
    let sub = Subscriber::new(Some(&connect_addr(&broker.message.xpub_bind))).expect("subscriber");
    sub.subscribe(topic).expect("subscribe");
    sub
}

fn publisher(broker: &RobotBusBroker) -> Publisher {
    let publ = Publisher::new(Some(&connect_addr(&broker.message.xsub_bind))).expect("publisher");
    thread::sleep(Duration::from_millis(100));
    publ
}

/// Publish until `sub` sees `payload` (demand needs a moment to travel between brokers).
fn publish_until_received(publ: &Publisher, topic: &str, sub: &Subscriber, payload: &[u8]) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        publ.publish(topic, payload).expect("publish");
        if let Ok((t, p)) = sub.receive(Some(Duration::from_millis(200))) {
            if p == payload {
                assert_eq!(t, topic);
                return;
            }
        }
    }
    panic!("{topic}: {payload:?} never arrived");
}

#[test]
fn joining_broker_reaches_running_broker_in_both_directions() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    // B runs first, with no peers, and is never reconfigured.
    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("broker b");
    thread::sleep(Duration::from_millis(100));
    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");

    let sub_a = subscriber(&broker_a, "fleet/pose");
    let sub_b = subscriber(&broker_b, "fleet/pose");
    let pub_a = publisher(&broker_a);
    let pub_b = publisher(&broker_b);

    publish_until_received(&pub_a, "fleet/pose", &sub_b, b"from-a");
    publish_until_received(&pub_b, "fleet/pose", &sub_a, b"from-b");

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn joined_broker_is_a_plain_proxy_until_someone_joins() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("broker b");
    let sub_b = subscriber(&broker_b, "local/topic");
    let pub_b = publisher(&broker_b);
    publish_until_received(&pub_b, "local/topic", &sub_b, b"before-join");

    // A joins while B's subscribers and publishers stay connected.
    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");
    let sub_a = subscriber(&broker_a, "local/topic");
    thread::sleep(Duration::from_millis(1500));

    publish_until_received(&pub_b, "local/topic", &sub_a, b"to-a");
    publish_until_received(&pub_b, "local/topic", &sub_b, b"still-local");

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn joined_messages_arrive_once_and_without_framing_artifacts() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("broker b");
    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");

    // Subscribers on both sides for the same topic: neither side may see echoes or
    // duplicates, and B's clients must see plain `[topic][payload]` messages.
    let sub_a = subscriber(&broker_a, "once");
    let sub_b = subscriber(&broker_b, "once");
    let pub_a = publisher(&broker_a);
    thread::sleep(Duration::from_millis(1500));

    pub_a.publish("once", b"unique").expect("publish");
    assert_eq!(count_matching(&sub_a, b"unique", Duration::from_millis(600)), 1);
    assert_eq!(count_matching(&sub_b, b"unique", Duration::from_millis(600)), 1);

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn configuring_both_sides_keeps_one_link() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");
    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", vec![join_peer("broker-a", a)], b))
            .expect("broker b");

    let sub_a = subscriber(&broker_a, "mesh");
    let sub_b = subscriber(&broker_b, "mesh");
    let pub_a = publisher(&broker_a);
    let pub_b = publisher(&broker_b);

    publish_until_received(&pub_a, "mesh", &sub_b, b"warmup-a");
    publish_until_received(&pub_b, "mesh", &sub_a, b"warmup-b");
    // Drain warmup duplicates from the retry loops above.
    while sub_a.receive(Some(Duration::from_millis(200))).is_ok() {}
    while sub_b.receive(Some(Duration::from_millis(200))).is_ok() {}

    pub_a.publish("mesh", b"from-a").expect("publish a");
    assert_eq!(count_matching(&sub_b, b"from-a", Duration::from_millis(600)), 1);
    assert_eq!(count_matching(&sub_a, b"from-a", Duration::from_millis(100)), 1);
    pub_b.publish("mesh", b"from-b").expect("publish b");
    assert_eq!(count_matching(&sub_a, b"from-b", Duration::from_millis(600)), 1);
    assert_eq!(count_matching(&sub_b, b"from-b", Duration::from_millis(100)), 1);

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn two_brokers_joining_one_hub_reach_each_other_through_it() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(3);
    let (a, b, c) = (&ports[0], &ports[1], &ports[2]);

    let hub =
        RobotBusBroker::start(federated_bus_config("hub", Vec::new(), b)).expect("hub broker");
    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("hub", b)], a))
            .expect("broker a");
    let broker_c =
        RobotBusBroker::start(federated_bus_config("broker-c", vec![join_peer("hub", b)], c))
            .expect("broker c");

    let sub_c = subscriber(&broker_c, "via/hub");
    let sub_hub = subscriber(&hub, "via/hub");
    let pub_a = publisher(&broker_a);

    publish_until_received(&pub_a, "via/hub", &sub_c, b"relayed");
    publish_until_received(&pub_a, "via/hub", &sub_hub, b"relayed-hub");

    broker_a.stop().expect("stop a");
    broker_c.stop().expect("stop c");
    hub.stop().expect("stop hub");
}

#[test]
fn reserved_topics_do_not_cross_a_join_link() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("broker b");
    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");

    let status = robot_bus::console_topics::STATUS;
    let sub_b = subscriber(&broker_b, status);
    sub_b.subscribe("fleet/pose").expect("subscribe");
    let pub_a = publisher(&broker_a);
    thread::sleep(Duration::from_millis(1500));

    pub_a.publish(status, b"status-a").expect("publish status");
    assert_eq!(count_matching(&sub_b, b"status-a", Duration::from_millis(600)), 0);
    publish_until_received(&pub_a, "fleet/pose", &sub_b, b"user-topic");

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn joined_broker_restart_is_rejoined_automatically() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("broker b");
    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");
    let sub_a = subscriber(&broker_a, "again");
    let pub_b = publisher(&broker_b);
    publish_until_received(&pub_b, "again", &sub_a, b"first");

    broker_b.stop().expect("stop b");
    thread::sleep(Duration::from_millis(200));
    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("restart b");
    let pub_b = publisher(&broker_b);

    // A's next HELLO re-creates the link and re-sends its demand; no restart of A.
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut rejoined = false;
    while Instant::now() < deadline && !rejoined {
        pub_b.publish("again", b"second").expect("publish");
        rejoined = matches!(
            sub_a.receive(Some(Duration::from_millis(200))),
            Ok((_, p)) if p == b"second"
        );
    }
    assert!(rejoined, "A did not rejoin the restarted B");

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn leaving_broker_leaves_no_demand_behind() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).expect("broker b");
    // A raw XPUB on B's XSUB sees exactly the subscriptions local publishers would see.
    let ctx = zmq::Context::new();
    let probe = ctx.socket(zmq::SocketType::XPUB).expect("probe");
    probe.set_xpub_verbose(true).expect("verbose");
    probe.connect(&broker_b.message.xsub_bind).expect("connect");

    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![join_peer("broker-b", b)], a))
            .expect("broker a");
    let sub_a = subscriber(&broker_a, "ghost");

    let wait_for = |want: &[u8], secs: u64| -> bool {
        let deadline = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < deadline {
            if probe.poll(zmq::POLLIN, 100).unwrap_or(0) > 0 {
                if let Ok(frame) = probe.recv_bytes(0) {
                    if frame == want {
                        return true;
                    }
                }
            }
        }
        false
    };
    assert!(wait_for(b"\x01ghost", 5), "B never saw A's demand");

    drop(sub_a);
    broker_a.stop().expect("stop a");
    assert!(
        wait_for(b"\x00ghost", 10),
        "B kept A's demand after A left"
    );

    broker_b.stop().expect("stop b");
}

#[test]
fn legacy_peers_still_work_with_mutual_configuration() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a =
        RobotBusBroker::start(federated_bus_config("broker-a", vec![msg_peer(b)], a))
            .expect("broker a");
    let broker_b =
        RobotBusBroker::start(federated_bus_config("broker-b", vec![msg_peer(a)], b))
            .expect("broker b");
    thread::sleep(Duration::from_millis(100));

    let sub_b = subscriber(&broker_b, "legacy");
    thread::sleep(Duration::from_millis(300));
    let pub_a = publisher(&broker_a);
    pub_a.publish("legacy", b"old-style").expect("publish");
    assert_eq!(
        recv_exact(&sub_b, b"old-style", Duration::from_secs(2)),
        "legacy"
    );

    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn transient_local_late_join_across_federated_brokers() {
    let _guard = lock_brokers();
    let ports = alloc_message_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);
    let broker_b = RobotBusBroker::start(federated_bus_config("broker-b", Vec::new(), b)).unwrap();
    let broker_a = RobotBusBroker::start(federated_bus_config(
        "broker-a",
        vec![join_peer("broker-b", b)],
        a,
    ))
    .unwrap();
    let qos = QosProfile::keep_last(4).transient_local();
    let mut writer = Node::with_options(
        "fleet-writer",
        NodeOptions {
            message_xsub: Some(connect_addr(&broker_a.message.xsub_bind)),
            ..NodeOptions::tcp()
        },
    );
    let publisher = writer
        .create_publisher_raw_with_qos("/fleet/state", qos)
        .unwrap();
    publisher.publish(b"current robot state").unwrap();
    let mut reader = Node::with_options(
        "fleet-reader",
        NodeOptions {
            message_xpub: Some(connect_addr(&broker_b.message.xpub_bind)),
            ..NodeOptions::tcp()
        },
    );
    let (tx, rx) = mpsc::channel();
    reader
        .create_subscription_raw_with_qos(
            "/fleet/state",
            qos,
            Arc::new(move |bytes| {
                let _ = tx.send(bytes.to_vec());
            }),
            None,
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        reader.spin_once(Some(Duration::from_millis(50))).unwrap();
        if let Ok(payload) = rx.try_recv() {
            assert_eq!(payload, b"current robot state");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "retained state did not cross brokers"
        );
    }
}
