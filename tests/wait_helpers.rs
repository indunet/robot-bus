//! wait_for_message / wait_for_service / wait_for_action_server helpers.

#![cfg(feature = "console")]

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::message_bus::Publisher;
use robot_bus::{Node, NodeOptions, RobotBusBroker, RobotBusConfig};
use support::{ephemeral_robot_bus_config, lock_brokers, MessageProxy};

fn console_on_config() -> RobotBusConfig {
    let mut config = ephemeral_robot_bus_config();
    config.console.enabled = true;
    config.console.tank_enabled = false;
    config
}

fn node_options_from_broker(broker: &RobotBusBroker) -> NodeOptions {
    NodeOptions {
        message_xsub: Some(broker.message.xsub_bind.clone()),
        message_xpub: Some(broker.message.xpub_bind.clone()),
        service_frontend: Some(broker.service.frontend_bind.clone()),
        service_backend: Some(broker.service.backend_bind.clone()),
        action_frontend: Some(broker.action.frontend_bind.clone()),
        action_backend: Some(broker.action.backend_bind.clone()),
        console_url: Some(broker.api_url()),
        ..NodeOptions::default()
    }
}

#[test]
fn wait_for_message_returns_payload() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    thread::sleep(Duration::from_millis(50));

    let options = NodeOptions {
        message_xsub: Some(proxy.xsub_endpoint.clone()),
        message_xpub: Some(proxy.xpub_endpoint.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("wait-msg", options);

    let topic = "/wait/demo";
    let pub_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        pub_.publish(topic, b"ping").expect("publish");
    });

    let got = node
        .wait_for_message(topic, Some(Duration::from_secs(2)))
        .expect("wait_for_message");
    assert_eq!(got.as_deref(), Some(&b"ping"[..]));
    pub_thread.join().unwrap();
}

#[test]
fn wait_for_message_times_out() {
    let proxy = MessageProxy::spawn();
    let options = NodeOptions {
        message_xsub: Some(proxy.xsub_endpoint.clone()),
        message_xpub: Some(proxy.xpub_endpoint.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("wait-msg-timeout", options);
    let got = node
        .wait_for_message("/missing", Some(Duration::from_millis(100)))
        .expect("wait_for_message");
    assert!(got.is_none());
}

#[test]
fn wait_for_service_ready_via_console_workers() {
    let _lock = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let server_opts = node_options_from_broker(&broker);
    let client_opts = server_opts.clone();

    let mut server = Node::with_options("svc-server", server_opts);
    server
        .create_service_raw(
            "/wait/echo",
            Arc::new(|body| body.to_vec()),
            None,
        )
        .expect("create_service");
    server.start().expect("start server");
    thread::sleep(Duration::from_millis(200));

    let mut client_node = Node::with_options("svc-client", client_opts);
    let client = client_node
        .create_client_raw("/wait/echo")
        .expect("create_client");
    assert!(
        client.wait_for_service(Some(Duration::from_secs(3))),
        "service should become ready"
    );
    assert!(client.service_is_ready());

    let missing = client_node
        .create_client_raw("/wait/missing")
        .expect("create_client missing");
    assert!(!missing.wait_for_service(Some(Duration::from_millis(200))));

    server.shutdown().ok();
    broker.stop().expect("stop");
}

#[test]
fn wait_for_action_server_ready_via_console_workers() {
    let _lock = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let server_opts = node_options_from_broker(&broker);
    let client_opts = server_opts.clone();

    let mut server = Node::with_options("act-server", server_opts);
    server
        .create_action_server_raw(
            "/wait/act",
            Arc::new(|_goal| vec![("RESULT".into(), b"ok".to_vec())]),
            None,
        )
        .expect("create_action_server");
    server.start().expect("start server");
    thread::sleep(Duration::from_millis(200));

    let mut client_node = Node::with_options("act-client", client_opts);
    let client = client_node
        .create_action_client_raw("/wait/act")
        .expect("create_action_client");
    assert!(
        client.wait_for_action_server(Some(Duration::from_secs(3))),
        "action server should become ready"
    );
    assert!(client.action_server_is_ready());

    server.shutdown().ok();
    broker.stop().expect("stop");
}
