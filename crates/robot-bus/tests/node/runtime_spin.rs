//! Executor / Node callback executor (subscribe + spin_once / spin / shutdown).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crate::common::MessageProxy;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::message_bus::{Publisher, Subscriber};
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::{
    CallbackGroup, Executor, HighWaterMark, MessageCallback, Node, NodeOptions, QosProfile,
    SingleThreadedExecutor,
};

#[test]
fn subscribe_callback_via_spin_once() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();
    let callback: MessageCallback = Arc::new(move |payload| {
        assert_eq!(payload, b"hello");
        hits_cb.fetch_add(1, Ordering::SeqCst);
    });

    let mut executor = Executor::new();
    executor
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect subscriber");
    executor
        .subscribe("demo.topic", callback, CallbackGroup::mutually_exclusive())
        .expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("demo.topic", b"hello").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for callback"
        );
        executor
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn spin_stops_on_shutdown() {
    let proxy = MessageProxy::spawn();
    let mut executor = Executor::new();
    executor
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect subscriber");
    executor
        .subscribe(
            "unused",
            Arc::new(|_| {}),
            CallbackGroup::mutually_exclusive(),
        )
        .expect("subscribe");

    let handle = executor.shutdown_handle();
    let joiner = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        handle.shutdown();
    });

    executor.spin().expect("spin");
    joiner.join().expect("joiner");
}

#[test]
fn spin_some_processes_pending_then_returns() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut executor = Executor::new();
    executor
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect subscriber");
    executor
        .subscribe(
            "demo.topic",
            Arc::new(move |_| {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
            CallbackGroup::mutually_exclusive(),
        )
        .expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("demo.topic", b"a").expect("publish");
    pub_.publish("demo.topic", b"b").expect("publish");

    executor
        .spin_some(Some(Duration::from_secs(2)))
        .expect("spin_some");
    assert!(hits.load(Ordering::SeqCst) >= 1);
}

#[test]
fn timer_fires_via_spin_once() {
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut executor = Executor::new();
    executor
        .create_timer(
            Duration::from_millis(40),
            Arc::new(move || {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
            CallbackGroup::mutually_exclusive(),
        )
        .expect("create_timer");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for timer"
        );
        executor
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert!(hits.load(Ordering::SeqCst) >= 1);
}

#[test]
fn cancel_timer_stops_firing() {
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut executor = Executor::new();
    let handle = executor
        .create_timer(
            Duration::from_millis(30),
            Arc::new(move || {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
            CallbackGroup::mutually_exclusive(),
        )
        .expect("create_timer");
    // Keep one active timer so the executor still has work to wait on.
    executor
        .create_timer(
            Duration::from_secs(60),
            Arc::new(|| {}),
            CallbackGroup::mutually_exclusive(),
        )
        .expect("keepalive timer");
    executor.cancel_timer(handle).expect("cancel");

    for _ in 0..5 {
        executor
            .spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[test]
fn publisher_subscriber_hwm_roundtrip() {
    let proxy = MessageProxy::spawn();
    let custom = HighWaterMark::new(16, 32);

    let pub_ = Publisher::with_hwm(Some(&proxy.xsub_endpoint), custom).expect("publisher");
    assert_eq!(pub_.high_water_mark().expect("get"), custom);

    let bumped = HighWaterMark::new(64, 64);
    pub_.set_high_water_mark(bumped).expect("set");
    assert_eq!(pub_.high_water_mark().expect("get"), bumped);

    let sub = Subscriber::with_hwm(Some(&proxy.xpub_endpoint), custom).expect("subscriber");
    assert_eq!(
        sub.high_water_mark().expect("get subscriber HWM"),
        HighWaterMark::new(0, custom.rcv)
    );
    sub.set_high_water_mark(bumped).expect("set subscriber HWM");
    assert_eq!(
        sub.high_water_mark().expect("get subscriber HWM"),
        HighWaterMark::new(0, bumped.rcv)
    );

    let mut executor = Executor::new();
    executor
        .set_stream_hwm(HighWaterMark::new(20, 20))
        .expect("set_stream_hwm");
    executor
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect");
    assert_eq!(executor.stream_hwm(), HighWaterMark::new(20, 20));
}

#[test]
fn node_topic_qos_keep_last_maps_to_hwm() {
    let proxy = MessageProxy::spawn();
    let (executor, mut node) = node_with_proxy("qos-pilot", &proxy);
    let _ = executor;

    let qos = QosProfile::keep_last(24);
    let pub_ = node
        .create_publisher_raw_with_qos("/qos/demo", qos)
        .expect("publisher with qos");
    assert_eq!(
        pub_.high_water_mark().expect("publisher hwm"),
        HighWaterMark::new(24, 24)
    );

    node.create_subscription_raw_with_qos(
        "/qos/demo",
        QosProfile::keep_last(32),
        Arc::new(|_| {}),
        None,
    )
    .expect("subscription with qos");
    assert_eq!(
        node.stream_hwm().expect("stream hwm"),
        HighWaterMark::new(32, 32)
    );
}

#[test]
fn node_tcp_subscriptions_register_at_any_qos_depth() {
    use std::time::Instant;

    let context = zmq::Context::new();
    let depths = [None, Some(1), Some(2), Some(4)];
    for depth_a in depths {
        for depth_b in depths {
            // Observe the SUB control frames directly instead of guessing when
            // subscriptions have propagated through a proxy.
            let xpub = context.socket(zmq::XPUB).expect("XPUB socket");
            xpub.bind("tcp://127.0.0.1:0").expect("bind XPUB");
            xpub.set_rcvtimeo(1000).expect("XPUB receive timeout");
            let options = NodeOptions {
                message_xpub: Some(xpub.get_last_endpoint().unwrap().unwrap()),
                ..NodeOptions::tcp()
            };
            let executor = SingleThreadedExecutor::new();
            let mut node = Node::with_options("qos-matrix", options);
            executor.add_node(&mut node).expect("add node");

            let mut subscriptions = Vec::new();
            for (topic, depth) in [("/t/a", depth_a), ("/t/b", depth_b)] {
                let hits = Arc::new(AtomicUsize::new(0));
                let count = Arc::clone(&hits);
                let callback: MessageCallback = Arc::new(move |payload| {
                    assert_eq!(payload, b"x");
                    count.fetch_add(1, Ordering::SeqCst);
                });
                match depth {
                    Some(depth) => node.create_subscription_raw_with_qos(
                        topic,
                        QosProfile::keep_last(depth),
                        callback,
                        None,
                    ),
                    None => node.create_subscription_raw(topic, callback, None),
                }
                .expect("create subscription");
                subscriptions.push((topic, hits));
            }

            for (topic, _) in &subscriptions {
                let frame = xpub.recv_bytes(0).unwrap_or_else(|err| {
                    panic!("QoS {depth_a:?}/{depth_b:?}: missing subscription for {topic}: {err}")
                });
                assert_eq!(frame[0], 1);
                assert_eq!(&frame[1..], topic.as_bytes());
            }

            for expected in 1..=5 {
                for (topic, hits) in &subscriptions {
                    // Let each callback drain before sending another message:
                    // this tests registration, not best-effort burst delivery.
                    xpub.send_multipart([topic.as_bytes(), b"x"], 0)
                        .expect("publish");
                    let deadline = Instant::now() + Duration::from_secs(1);
                    while hits.load(Ordering::SeqCst) < expected {
                        assert!(
                            Instant::now() < deadline,
                            "QoS {depth_a:?}/{depth_b:?}: missing message {expected} on {topic}"
                        );
                        executor.spin_once(Some(Duration::from_millis(20))).unwrap();
                    }
                    assert_eq!(hits.load(Ordering::SeqCst), expected);
                }
            }
        }
    }
}

fn node_with_proxy(name: &str, proxy: &MessageProxy) -> (SingleThreadedExecutor, Node) {
    let options = NodeOptions {
        message_xsub: Some(proxy.xsub_endpoint.clone()),
        message_xpub: Some(proxy.xpub_endpoint.clone()),
        ..NodeOptions::default()
    };
    let executor = SingleThreadedExecutor::new();
    let mut node = Node::with_options(name, options);
    executor.add_node(&mut node).expect("add_node");
    (executor, node)
}

#[test]
fn node_subscription_uses_topic_as_given() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();
    let callback: MessageCallback = Arc::new(move |payload| {
        assert_eq!(payload, b"hello");
        hits_cb.fetch_add(1, Ordering::SeqCst);
    });

    let (executor, mut node) = node_with_proxy("pilot", &proxy);
    node.create_subscription_raw("/robot1/imu", callback, None)
        .expect("create_subscription_raw");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("/robot1/imu", b"hello").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for node callback"
        );
        executor
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn node_publish_uses_topic_as_given() {
    let proxy = MessageProxy::spawn();
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let (executor, mut sub_node) = node_with_proxy("listener", &proxy);
    sub_node
        .create_subscription_raw(
            "/robot1/cmd_vel",
            Arc::new(move |payload| {
                assert_eq!(payload, b"go");
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
            None,
        )
        .expect("create_subscription_raw");
    thread::sleep(Duration::from_millis(150));

    let (_pub_exec, mut pub_node) = node_with_proxy("pilot", &proxy);
    let publisher = pub_node
        .create_publisher_raw("/robot1/cmd_vel")
        .expect("create_publisher_raw");
    publisher.publish(b"go").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for node publish"
        );
        executor
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn node_timer_via_spin_once() {
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let executor = SingleThreadedExecutor::new();
    let mut node = Node::new("timer_node");
    executor.add_node(&mut node).expect("add_node");
    node.create_timer(
        Duration::from_millis(40),
        Arc::new(move || {
            hits_cb.fetch_add(1, Ordering::SeqCst);
        }),
        None,
    )
    .expect("create_timer");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for node timer"
        );
        executor
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert!(hits.load(Ordering::SeqCst) >= 1);
}

#[test]
fn node_subscription_typed_imu() {
    let proxy = MessageProxy::spawn();
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let (executor, mut node) = node_with_proxy("imu_node", &proxy);
    node.create_subscription::<Imu, _>(
        "/robot1/imu",
        move |imu| {
            // Only count the intentionally valid sample (bad frames are skipped or default).
            if imu.linear_acceleration.as_ref().map(|v| v.z) == Some(9.8) {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }
        },
        None,
    )
    .expect("create_subscription");
    thread::sleep(Duration::from_millis(150));

    let imu = Imu {
        linear_acceleration: Some(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 9.8,
        }),
        ..Default::default()
    };
    let (_pub_exec, mut pub_node) = node_with_proxy("imu_pub", &proxy);
    let imu_pub = pub_node
        .create_publisher::<Imu>("/robot1/imu")
        .expect("create_publisher");
    imu_pub.publish(&imu).expect("publish");
    // Truncated/invalid protobuf varint — decode should fail and be skipped.
    let raw_pub = pub_node
        .create_publisher_raw("/robot1/imu")
        .expect("create_publisher_raw");
    raw_pub
        .publish(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01])
        .expect("publish bad");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for typed callback"
        );
        executor
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    // Drain a bit more so a bad decode would have a chance to mis-count.
    for _ in 0..3 {
        executor
            .spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn node_spin_without_explicit_executor() {
    let proxy = MessageProxy::spawn();
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let options = NodeOptions {
        message_xsub: Some(proxy.xsub_endpoint.clone()),
        message_xpub: Some(proxy.xpub_endpoint.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("auto_exec", options);
    // No add_node — Node lazily owns a SingleThreadedExecutor.
    node.create_subscription_raw(
        "/auto/topic",
        Arc::new(move |payload| {
            assert_eq!(payload, b"ping");
            hits_cb.fetch_add(1, Ordering::SeqCst);
        }),
        None,
    )
    .expect("create_subscription_raw");
    thread::sleep(Duration::from_millis(150));

    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    pub_.publish("/auto/topic", b"ping").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for auto-executor callback"
        );
        node.spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}
