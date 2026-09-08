//! Diagnostics and bus error propagation without installing ROS.
mod support;

use robot_bus::errors::{self, BusError, parse_error_body, rpc_error_body};
#[allow(dead_code)]
#[path = "../src/ros2_bridge/drop_stats.rs"]
mod health;

use robot_bus::{Node, NodeOptions, RobotBusBroker};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn bridge_errors_roundtrip_without_changing_success_payloads() {
    for error in [
        BusError::ActionAborted("stopped".into()),
        BusError::ActionRejected("invalid goal".into()),
        BusError::Cancelled {
            name: "motion".into(),
        },
        BusError::Timeout("late".into()),
        BusError::Protocol("decode failed".into()),
    ] {
        let decoded = parse_error_body(&rpc_error_body(&error)).expect("error body");
        assert_eq!(
            std::mem::discriminant(&decoded),
            std::mem::discriminant(&error)
        );
        assert_eq!(decoded.to_string(), error.to_string());
    }
    assert!(parse_error_body(b"ordinary result").is_none());
    assert!(parse_error_body(b"ACTION_ABORTED without delimiter").is_none());
}

#[test]
fn action_client_receives_bridge_failures_as_errors() {
    let _guard = support::lock_brokers();
    let broker = RobotBusBroker::start(support::ephemeral_robot_bus_config()).unwrap();
    let options = NodeOptions {
        action_frontend: Some(broker.action.frontend_bind.clone()),
        action_backend: Some(broker.action.backend_bind.clone()),
        ..NodeOptions::default()
    };
    let mut server = Node::with_options("bridge_error_server", options.clone());
    server
        .create_action_server_raw_live("bridge_outcome", Arc::new(|body, _| body.to_vec()), None)
        .unwrap();
    let mut client_node = Node::with_options("bridge_error_client", options);
    let client = client_node
        .create_action_client_raw("bridge_outcome")
        .unwrap();
    #[allow(unused_mut)]
    let mut clients = vec![client];
    #[cfg(feature = "ws")]
    let mut ws_node = Node::ws_at(
        "bridge_ws_client",
        format!("http://{}", broker.api_listen()),
    );
    #[cfg(feature = "ws")]
    clients.push(ws_node.create_action_client_raw("bridge_outcome").unwrap());
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        for client in clients {
            for error in [
                BusError::ActionAborted("stopped".into()),
                BusError::ActionRejected("invalid".into()),
                BusError::Cancelled {
                    name: "motion".into(),
                },
                BusError::Timeout("late".into()),
            ] {
                let goal = client
                    .send_goal(
                        &rpc_error_body(&error),
                        None,
                        Some(Duration::from_secs(2)),
                        None,
                    )
                    .unwrap();
                let got = goal
                    .wait_result()
                    .expect_err("must not return default success");
                assert_eq!(std::mem::discriminant(&got), std::mem::discriminant(&error));
            }
            let goal = client
                .send_goal(b"success", None, Some(Duration::from_secs(2)), None)
                .unwrap();
            assert_eq!(goal.wait_result().unwrap().body, b"success");
        }
    });
    let deadline = Instant::now() + Duration::from_secs(12);
    while !worker.is_finished() {
        assert!(Instant::now() < deadline, "bridge error test timed out");
        server.spin_once(Some(Duration::from_millis(10))).unwrap();
    }
    worker.join().unwrap();
    broker.stop().unwrap();
}
