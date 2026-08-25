//! wait_for_message / wait_for_service / wait_for_action_server helpers.

#![cfg(feature = "console")]

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{
    ConsoleBrokerConfig, DiscoveryConfig, WsGatewayConfig, RobotBusBroker, RobotBusConfig,
};
use robot_bus::message_bus::Publisher;
use robot_bus::{Node, NodeOptions};
use support::{free_port, lock_brokers, MessageProxy};

fn test_broker_config(
    msg_xsub: u16,
    msg_xpub: u16,
    svc_fe: u16,
    svc_be: u16,
    act_fe: u16,
    act_be: u16,
    http: u16,
) -> RobotBusConfig {
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{msg_xsub}"),
            xpub_bind: format!("tcp://127.0.0.1:{msg_xpub}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{svc_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{svc_be}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{act_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{act_be}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ActionBusConfig::default()
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        ws: WsGatewayConfig {
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            ..WsGatewayConfig::default()
        },
        console: ConsoleBrokerConfig {
            enabled: true,
            tank_enabled: false,
            docs_enabled: true,
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            cors_origins: vec![],
        },
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
    let ports: [u16; 7] = std::array::from_fn(|_| free_port());
    let broker = RobotBusBroker::start(test_broker_config(
        ports[0], ports[1], ports[2], ports[3], ports[4], ports[5], ports[6],
    ))
    .expect("broker");
    let console_url = format!("http://127.0.0.1:{}", ports[6]);

    let server_opts = NodeOptions {
        message_xsub: Some(format!("tcp://127.0.0.1:{}", ports[0])),
        message_xpub: Some(format!("tcp://127.0.0.1:{}", ports[1])),
        service_frontend: Some(format!("tcp://127.0.0.1:{}", ports[2])),
        service_backend: Some(format!("tcp://127.0.0.1:{}", ports[3])),
        action_frontend: Some(format!("tcp://127.0.0.1:{}", ports[4])),
        action_backend: Some(format!("tcp://127.0.0.1:{}", ports[5])),
        console_url: Some(console_url.clone()),
        ..NodeOptions::default()
    };
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
    let ports: [u16; 7] = std::array::from_fn(|_| free_port());
    let broker = RobotBusBroker::start(test_broker_config(
        ports[0], ports[1], ports[2], ports[3], ports[4], ports[5], ports[6],
    ))
    .expect("broker");
    let console_url = format!("http://127.0.0.1:{}", ports[6]);

    let server_opts = NodeOptions {
        message_xsub: Some(format!("tcp://127.0.0.1:{}", ports[0])),
        message_xpub: Some(format!("tcp://127.0.0.1:{}", ports[1])),
        service_frontend: Some(format!("tcp://127.0.0.1:{}", ports[2])),
        service_backend: Some(format!("tcp://127.0.0.1:{}", ports[3])),
        action_frontend: Some(format!("tcp://127.0.0.1:{}", ports[4])),
        action_backend: Some(format!("tcp://127.0.0.1:{}", ports[5])),
        console_url: Some(console_url),
        ..NodeOptions::default()
    };
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
