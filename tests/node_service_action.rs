//! Node-based service / action client round-trips (ROS 2–style API).

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::action_bus::ActionKind;
use robot_bus::action::v1::{
    Fibonacci, FibonacciFeedback, FibonacciGoal, FibonacciResult,
};
use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest, SetBoolResponse};
use robot_bus::{ActionOutcome, Node, NodeOptions, RobotBusBroker, SingleThreadedExecutor};
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
fn node_service_client_echo_raw() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("svc_server", options.clone());
        let mut client_node = Node::with_options("svc_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_service_raw(
                "echo",
                Arc::new(|body| [b"echo:", body].concat()),
                None,
            )
            .expect("create_service_raw");

        let client = client_node
            .create_client_raw("echo")
            .expect("create_client_raw");
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
    }

    broker.stop().expect("stop broker");
}

#[test]
fn node_service_client_set_bool_typed() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("svc_server", options.clone());
        let mut client_node = Node::with_options("svc_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_service::<SetBool, _>(
                "/set_bool",
                |req: SetBoolRequest| SetBoolResponse {
                    success: true,
                    message: format!("set:{}", req.data),
                },
                None,
            )
            .expect("create_service");

        let client = client_node
            .create_client::<SetBool>("/set_bool")
            .expect("create_client");
        let handle = executor.shutdown_handle().expect("shutdown handle");

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let resp = client
                .call(
                    &SetBoolRequest { data: true },
                    Some(Duration::from_secs(5)),
                )
                .expect("call");
            assert!(resp.success);
            assert_eq!(resp.message, "set:true");
            handle.shutdown();
        });

        executor.spin().expect("spin");
    }

    broker.stop().expect("stop broker");
}

#[test]
fn node_action_client_goal_raw() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("act_server", options.clone());
        let mut client_node = Node::with_options("act_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_action_server_raw(
                "demo",
                Arc::new(|body| {
                    vec![
                        ("FEEDBACK".into(), b"step-1".to_vec()),
                        ("RESULT".into(), [b"done:", body].concat()),
                    ]
                }),
                None,
            )
            .expect("create_action_server_raw");

        let client = client_node
            .create_action_client_raw("demo")
            .expect("create_action_client_raw");
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
fn node_action_client_fibonacci_typed() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("act_server", options.clone());
        let mut client_node = Node::with_options("act_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_action_server::<Fibonacci, _>(
                "fibonacci",
                |goal: FibonacciGoal| {
                    let order = goal.order.max(0) as usize;
                    let mut seq = Vec::with_capacity(order);
                    for i in 0..order {
                        if i < 2 {
                            seq.push(i as i32);
                        } else {
                            seq.push(seq[i - 1] + seq[i - 2]);
                        }
                    }
                    let feedback_seq = if seq.len() > 1 {
                        seq[..seq.len() - 1].to_vec()
                    } else {
                        seq.clone()
                    };
                    ActionOutcome {
                        feedbacks: vec![FibonacciFeedback {
                            sequence: feedback_seq,
                        }],
                        result: FibonacciResult { sequence: seq },
                    }
                },
                None,
            )
            .expect("create_action_server");

        let client = client_node
            .create_action_client::<Fibonacci>("fibonacci")
            .expect("create_action_client");
        let handle = executor.shutdown_handle().expect("shutdown handle");

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let outcome = client
                .send_goal(&FibonacciGoal { order: 5 }, None, Some(Duration::from_secs(10)))
                .expect("send_goal");
            assert_eq!(outcome.result.sequence, vec![0, 1, 1, 2, 3]);
            assert!(!outcome.feedbacks.is_empty());
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
