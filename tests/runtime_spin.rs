//! BusRuntime / Node callback executor (subscribe + spin_once / spin / shutdown).

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::message_bus::Publisher;
use robot_bus::{BusRuntime, HighWaterMark, MessageCallback, Node};
use support::MessageProxy;

#[test]
fn subscribe_callback_via_spin_once() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();
    let callback: MessageCallback = Arc::new(move |topic, payload| {
        assert_eq!(topic, "demo.topic");
        assert_eq!(payload, b"hello");
        hits_cb.fetch_add(1, Ordering::SeqCst);
    });

    let mut runtime = BusRuntime::new();
    runtime
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect subscriber");
    runtime
        .subscribe("demo.topic", callback)
        .expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("demo.topic", b"hello").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for callback"
        );
        runtime
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn spin_stops_on_shutdown() {
    let proxy = MessageProxy::spawn();
    let mut runtime = BusRuntime::new();
    runtime
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect subscriber");
    runtime
        .subscribe(
            "unused",
            Arc::new(|_topic, _payload| {}),
        )
        .expect("subscribe");

    let handle = runtime.shutdown_handle();
    let joiner = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        handle.shutdown();
    });

    runtime.spin().expect("spin");
    joiner.join().expect("joiner");
}

#[test]
fn spin_some_processes_pending_then_returns() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut runtime = BusRuntime::new();
    runtime
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect subscriber");
    runtime
        .subscribe(
            "demo.topic",
            Arc::new(move |_topic, _payload| {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("demo.topic", b"a").expect("publish");
    pub_.publish("demo.topic", b"b").expect("publish");

    runtime
        .spin_some(Some(Duration::from_secs(2)))
        .expect("spin_some");
    assert!(hits.load(Ordering::SeqCst) >= 1);
}

#[test]
fn timer_fires_via_spin_once() {
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut runtime = BusRuntime::new();
    runtime
        .create_timer(
            Duration::from_millis(40),
            Arc::new(move || {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .expect("create_timer");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for timer"
        );
        runtime
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert!(hits.load(Ordering::SeqCst) >= 1);
}

#[test]
fn cancel_timer_stops_firing() {
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut runtime = BusRuntime::new();
    let handle = runtime
        .create_timer(
            Duration::from_millis(30),
            Arc::new(move || {
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .expect("create_timer");
    // Keep one active timer so the executor still has work to wait on.
    runtime
        .create_timer(Duration::from_secs(60), Arc::new(|| {}))
        .expect("keepalive timer");
    runtime.cancel_timer(handle).expect("cancel");

    for _ in 0..5 {
        runtime
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

    let mut runtime = BusRuntime::new();
    runtime
        .set_stream_hwm(HighWaterMark::new(20, 20))
        .expect("set_stream_hwm");
    runtime
        .connect_subscriber(Some(&proxy.xpub_endpoint))
        .expect("connect");
    assert_eq!(runtime.stream_hwm(), HighWaterMark::new(20, 20));
}

#[test]
fn node_subscription_applies_namespace() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();
    let callback: MessageCallback = Arc::new(move |topic, payload| {
        assert_eq!(topic, "robot1/imu");
        assert_eq!(payload, b"hello");
        hits_cb.fetch_add(1, Ordering::SeqCst);
    });

    let mut node = Node::with_namespace("pilot", "robot1");
    node.create_subscription("imu", callback, Some(&proxy.xpub_endpoint))
        .expect("create_subscription");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("robot1/imu", b"hello").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for node callback"
        );
        node.spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn node_publish_applies_namespace() {
    let proxy = MessageProxy::spawn();
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut sub_node = Node::new("listener");
    sub_node
        .create_subscription(
            "robot1/cmd_vel",
            Arc::new(move |topic, payload| {
                assert_eq!(topic, "robot1/cmd_vel");
                assert_eq!(payload, b"go");
                hits_cb.fetch_add(1, Ordering::SeqCst);
            }),
            Some(&proxy.xpub_endpoint),
        )
        .expect("create_subscription");
    thread::sleep(Duration::from_millis(150));

    let mut pub_node = Node::with_namespace("pilot", "robot1");
    pub_node
        .create_publisher(Some(&proxy.xsub_endpoint))
        .expect("create_publisher");
    pub_node.publish("cmd_vel", b"go").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for namespaced publish"
        );
        sub_node
            .spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn node_timer_via_spin_once() {
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = hits.clone();

    let mut node = Node::new("timer_node");
    node.create_timer(
        Duration::from_millis(40),
        Arc::new(move || {
            hits_cb.fetch_add(1, Ordering::SeqCst);
        }),
    )
    .expect("create_timer");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(
            deadline > std::time::Instant::now(),
            "timed out waiting for node timer"
        );
        node.spin_once(Some(Duration::from_millis(100)))
            .expect("spin_once");
    }
    assert!(hits.load(Ordering::SeqCst) >= 1);
}
