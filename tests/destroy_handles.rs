//! Destroy subscription / service / action server handles.

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use robot_bus::message_bus::Publisher;
use robot_bus::{MessageCallback, Node, NodeOptions, RobotBusBroker};
use support::{MessageProxy, ephemeral_robot_bus_config, lock_brokers};

fn node_options_from_broker(broker: &RobotBusBroker) -> NodeOptions {
    NodeOptions {
        message_xsub: Some(broker.message.xsub_bind.clone()),
        message_xpub: Some(broker.message.xpub_bind.clone()),
        service_frontend: Some(broker.service.frontend_bind.clone()),
        service_backend: Some(broker.service.backend_bind.clone()),
        action_frontend: Some(broker.action.frontend_bind.clone()),
        action_backend: Some(broker.action.backend_bind.clone()),
        ..NodeOptions::default()
    }
}

#[test]
fn destroy_subscription_stops_callbacks() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = Arc::clone(&hits);
    let cb: MessageCallback = Arc::new(move |_| {
        hits_cb.fetch_add(1, Ordering::SeqCst);
    });

    let options = NodeOptions {
        message_xsub: Some(proxy.xsub_endpoint.clone()),
        message_xpub: Some(proxy.xpub_endpoint.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("destroy-sub", options);
    let handle = node
        .create_subscription_raw("/destroy/topic", cb, None)
        .expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("/destroy/topic", b"a").expect("publish");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hits.load(Ordering::SeqCst) == 0 {
        assert!(deadline > std::time::Instant::now(), "timeout first msg");
        let _ = node.spin_once(Some(Duration::from_millis(50)));
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    node.destroy_subscription(handle).expect("destroy");
    pub_.publish("/destroy/topic", b"b").expect("publish");
    for _ in 0..20 {
        let _ = node.spin_once(Some(Duration::from_millis(50)));
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn destroy_one_of_two_subscriptions_on_same_topic() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let a = Arc::new(AtomicUsize::new(0));
    let b = Arc::new(AtomicUsize::new(0));
    let a_cb = Arc::clone(&a);
    let b_cb = Arc::clone(&b);

    let options = NodeOptions {
        message_xsub: Some(proxy.xsub_endpoint.clone()),
        message_xpub: Some(proxy.xpub_endpoint.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("destroy-multi", options);
    let ha = node
        .create_subscription_raw(
            "/multi",
            Arc::new(move |_| {
                a_cb.fetch_add(1, Ordering::SeqCst);
            }),
            None,
        )
        .expect("sub a");
    let _hb = node
        .create_subscription_raw(
            "/multi",
            Arc::new(move |_| {
                b_cb.fetch_add(1, Ordering::SeqCst);
            }),
            None,
        )
        .expect("sub b");
    thread::sleep(Duration::from_millis(150));

    pub_.publish("/multi", b"1").expect("publish");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while a.load(Ordering::SeqCst) == 0 || b.load(Ordering::SeqCst) == 0 {
        assert!(deadline > std::time::Instant::now(), "timeout both");
        let _ = node.spin_once(Some(Duration::from_millis(50)));
    }

    node.destroy_subscription(ha).expect("destroy a");
    a.store(0, Ordering::SeqCst);
    b.store(0, Ordering::SeqCst);
    pub_.publish("/multi", b"2").expect("publish");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while b.load(Ordering::SeqCst) == 0 {
        assert!(deadline > std::time::Instant::now(), "timeout b only");
        let _ = node.spin_once(Some(Duration::from_millis(50)));
    }
    assert_eq!(a.load(Ordering::SeqCst), 0);
    assert_eq!(b.load(Ordering::SeqCst), 1);
}

#[test]
fn destroy_service_makes_client_fail() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    let mut server = Node::with_options("svc-server", options.clone());
    let svc = server
        .create_service_raw("/destroy/echo", Arc::new(|body| body.to_vec()), None)
        .expect("create_service");

    let mut client_node = Node::with_options("svc-client", options.clone());
    let client = client_node
        .create_client_raw("/destroy/echo")
        .expect("client");
    let stop = server.shutdown_handle().expect("shutdown handle");
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let reply = client
            .call(b"hi", Some(Duration::from_secs(2)))
            .expect("call before destroy");
        assert_eq!(reply, b"hi");
        stop.shutdown();
    });
    server.spin().expect("spin");

    server.destroy_service(&svc).expect("destroy_service");
    thread::sleep(Duration::from_millis(200));

    let client2 = client_node
        .create_client_raw("/destroy/echo")
        .expect("client2");
    let err = client2
        .call(b"hi", Some(Duration::from_millis(500)))
        .expect_err("call after destroy");
    let msg = err.to_string();
    assert!(
        msg.contains("NO_WORKER")
            || msg.to_ascii_lowercase().contains("timeout")
            || msg.to_ascii_lowercase().contains("timed out"),
        "unexpected error: {msg}"
    );

    broker.stop().expect("stop");
}

#[test]
fn destroy_action_server_makes_client_fail() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    let mut server = Node::with_options("act-server", options.clone());
    let act = server
        .create_action_server_raw(
            "/destroy/act",
            Arc::new(|_| vec![("RESULT".into(), b"ok".to_vec())]),
            None,
        )
        .expect("create_action_server");

    let mut client_node = Node::with_options("act-client", options);
    let client = client_node
        .create_action_client_raw("/destroy/act")
        .expect("action client");
    let stop = server.shutdown_handle().expect("shutdown handle");
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let outcome = client
            .send_goal_and_wait(b"g", None, Some(Duration::from_secs(2)))
            .expect("goal before destroy");
        assert!(!outcome.is_empty());
        stop.shutdown();
    });
    server.spin().expect("spin");

    server
        .destroy_action_server(&act)
        .expect("destroy_action_server");
    thread::sleep(Duration::from_millis(200));

    let client2 = client_node
        .create_action_client_raw("/destroy/act")
        .expect("action client2");
    let err = client2
        .send_goal_and_wait(b"g", None, Some(Duration::from_millis(500)))
        .expect_err("goal after destroy");
    let msg = err.to_string();
    assert!(
        msg.contains("NO_WORKER")
            || msg.to_ascii_lowercase().contains("timeout")
            || msg.to_ascii_lowercase().contains("timed out"),
        "unexpected error: {msg}"
    );

    broker.stop().expect("stop");
}
