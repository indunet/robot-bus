//! Node-based service / action client round-trips (ROS 2–style API).

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::action_bus::ActionKind;
use robot_bus::{Node, NodeOptions, RobotBusBroker, SingleThreadedExecutor};
use support::{ephemeral_robot_bus_config, lock_brokers};

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
fn node_service_client_echo() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("svc_server", options.clone());
        let mut client_node = Node::with_options("svc_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_service(
                "echo",
                Arc::new(|_client_id, _req_id, body| [b"echo:", body].concat()),
                None,
                None,
            )
            .expect("create_service");

        let client = client_node.create_client("echo").expect("create_client");
        let handle = executor.shutdown_handle().expect("shutdown handle");

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let reply = client
                .call(b"ping", Some(Duration::from_secs(5)))
                .expect("call");
            assert_eq!(reply, b"echo:ping");
            handle.shutdown();
        });

        executor.spin().expect("spin");
        // Drop node/executor while broker is still up so worker DISCONNECT can send.
    }

    broker.stop().expect("stop broker");
}

#[test]
fn node_action_client_goal() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("act_server", options.clone());
        let mut client_node = Node::with_options("act_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_action(
                "demo",
                Arc::new(|_client_id, _goal_id, body| {
                    vec![
                        ("FEEDBACK".into(), b"step-1".to_vec()),
                        ("RESULT".into(), [b"done:", body].concat()),
                    ]
                }),
                None,
                None,
            )
            .expect("create_action");

        let client = client_node
            .create_action_client("demo")
            .expect("create_action_client");
        let handle = executor.shutdown_handle().expect("shutdown handle");

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let messages = client
                .send_goal(b"fly", None, Some(Duration::from_secs(10)))
                .expect("send_goal");
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].kind, ActionKind::Feedback);
            assert_eq!(messages[0].body, b"step-1");
            assert_eq!(messages[1].kind, ActionKind::Result);
            assert_eq!(messages[1].body, b"done:fly");
            handle.shutdown();
        });

        executor.spin().expect("spin");
    }

    broker.stop().expect("stop broker");
}

#[test]
fn service_frontend_option_override() {
    let opts = NodeOptions {
        service_frontend: Some("tcp://127.0.0.1:19999".into()),
        ..NodeOptions::default()
    };
    assert_eq!(
        opts.service_frontend_endpoint().unwrap(),
        "tcp://127.0.0.1:19999"
    );
}
