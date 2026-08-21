//! Node broker session: connection_state and wait_for_broker.

#[cfg(feature = "ws")]
mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use robot_bus::{ConnectionState, Node, NodeOptions};

#[cfg(feature = "ws")]
use robot_bus::RobotBusBroker;
#[cfg(feature = "ws")]
use support::{ephemeral_robot_bus_config, lock_brokers};

#[test]
fn tcp_node_constructs_without_broker() {
    let mut opts = NodeOptions::tcp();
    opts.console_url = Some("http://127.0.0.1:1".into());
    let mut node = Node::with_options("session-offline", opts);
    assert_ne!(node.connection_state(), ConnectionState::Connected);
    assert_ne!(node.connection_state(), ConnectionState::Shutdown);
    assert!(!node.wait_for_broker(Some(Duration::from_millis(200))));
    node.shutdown().expect("shutdown");
    assert_eq!(node.connection_state(), ConnectionState::Shutdown);
}

#[test]
fn inproc_node_is_connected_without_http() {
    let node = Node::inproc("session-inproc");
    assert!(node.wait_for_broker(Some(Duration::from_secs(1))));
    assert_eq!(node.connection_state(), ConnectionState::Connected);
}

#[test]
fn shutdown_emits_connection_event() {
    let mut node = Node::tcp("session-events");
    // Point discover at a closed port so a leftover local broker cannot connect.
    // (Node::tcp already started; events still fire on shutdown.)
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen_cb = Arc::clone(&seen);
    node.add_on_connection_event(move |_old, new, _reason| {
        seen_cb.lock().unwrap().push(new.as_str().to_string());
    });
    node.shutdown().expect("shutdown");
    assert!(
        seen.lock().unwrap().iter().any(|s| s == "shutdown"),
        "events={:?}",
        seen.lock().unwrap()
    );
}

/// HTTP discover needs the API gateway (`ws` feature).
#[cfg(feature = "ws")]
#[test]
fn wait_for_broker_after_late_start() {
    let _guard = lock_brokers();
    let api_port = support::free_port();
    let mut opts = NodeOptions::tcp();
    opts.console_url = Some(format!("http://127.0.0.1:{api_port}"));
    let node = Node::with_options("session-late", opts);
    assert!(!node.wait_for_broker(Some(Duration::from_millis(250))));

    let mut config = ephemeral_robot_bus_config();
    config.ws.listen = format!("127.0.0.1:{api_port}").parse().unwrap();
    #[cfg(feature = "console")]
    {
        config.console.enabled = false;
    }
    let broker = RobotBusBroker::start(config).expect("start broker");
    assert!(
        node.wait_for_broker(Some(Duration::from_secs(5))),
        "state={}",
        node.connection_state()
    );
    assert_eq!(node.connection_state(), ConnectionState::Connected);
    broker.stop().expect("stop");
}
