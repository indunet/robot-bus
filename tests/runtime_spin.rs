//! BusRuntime callback executor (subscribe + spin_once / spin / shutdown).

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::message_bus::Publisher;
use robot_bus::{BusRuntime, MessageCallback};
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
