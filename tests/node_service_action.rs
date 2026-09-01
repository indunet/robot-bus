//! Node-based service / action client round-trips (ROS 2–style API).

mod support;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::action_bus::ActionKind;
use robot_bus::example_interfaces::action::v1::{
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
            .create_service_raw("echo", Arc::new(|body| [b"echo:", body].concat()), None)
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
                .call(&SetBoolRequest { data: true }, Some(Duration::from_secs(5)))
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
            let feedbacks = Arc::new(std::sync::Mutex::new(Vec::new()));
            let callback_feedbacks = Arc::clone(&feedbacks);
            let goal = client
                .send_goal(
                    b"fly",
                    None,
                    Some(Duration::from_secs(10)),
                    Some(Arc::new(move |message: &robot_bus::ActionMessage| {
                        callback_feedbacks
                            .lock()
                            .expect("feedback mutex")
                            .push(message.body.clone());
                    })),
                )
                .expect("send_goal");
            assert_eq!(goal.action_name(), "demo");
            assert!(!goal.goal_id().is_empty());
            let messages = goal.collect().expect("collect");
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].kind, ActionKind::Feedback);
            assert_eq!(messages[0].body, b"step-1");
            assert_eq!(messages[1].kind, ActionKind::Result);
            assert_eq!(messages[1].body, b"done:fly");
            assert_eq!(
                *feedbacks.lock().expect("feedback mutex"),
                vec![b"step-1".to_vec()]
            );
            handle.shutdown();
        });

        executor.spin().expect("spin");
    }

    broker.stop().expect("stop broker");
}

#[test]
fn node_action_live_feedback_arrives_before_result() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("act_server", options.clone());
        let mut client_node = Node::with_options("act_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_action_server_raw_live(
                "stream",
                Arc::new(|_body, ctx| {
                    ctx.publish_feedback(b"fb0");
                    thread::sleep(Duration::from_millis(300));
                    ctx.publish_feedback(b"fb1");
                    thread::sleep(Duration::from_millis(300));
                    ctx.publish_feedback(b"fb2");
                    b"done".to_vec()
                }),
                None,
            )
            .expect("create_action_server_raw_live");

        let client = client_node
            .create_action_client_raw("stream")
            .expect("create_action_client_raw");
        let handle = executor.shutdown_handle().expect("shutdown handle");

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let first_fb = Arc::new(std::sync::Mutex::new(None::<Instant>));
            let stamp_fb = Arc::clone(&first_fb);
            let goal = client
                .send_goal(
                    b"go",
                    None,
                    Some(Duration::from_secs(10)),
                    Some(Arc::new(move |message: &robot_bus::ActionMessage| {
                        if message.kind == ActionKind::Feedback {
                            let mut slot = stamp_fb.lock().expect("fb stamp");
                            if slot.is_none() {
                                *slot = Some(Instant::now());
                            }
                        }
                    })),
                )
                .expect("send_goal");
            let result = goal.wait_result().expect("wait_result");
            let t_res = Instant::now();
            assert_eq!(result.body, b"done");
            let t_fb = first_fb.lock().expect("fb").expect("feedback arrived");
            assert!(
                t_res.saturating_duration_since(t_fb) >= Duration::from_millis(200),
                "feedback must reach the client before RESULT is flushed (live, not batched)"
            );
            handle.shutdown();
        });

        executor.spin().expect("spin");
    }

    broker.stop().expect("stop broker");
}

#[test]
fn node_action_cancel_interrupts_inflight_goal() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    {
        let mut server = Node::with_options("act_server", options.clone());
        let mut client_node = Node::with_options("act_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        let took_cancel = Arc::new(AtomicBool::new(false));
        let took_cancel_h = Arc::clone(&took_cancel);
        server
            .create_action_server_raw_live(
                "cancellable",
                Arc::new(move |_body, ctx| {
                    ctx.publish_feedback(b"started");
                    for _ in 0..200 {
                        if ctx.cancel_requested() {
                            took_cancel_h.store(true, Ordering::SeqCst);
                            return b"cancelled".to_vec();
                        }
                        thread::sleep(Duration::from_millis(20));
                    }
                    b"completed".to_vec()
                }),
                None,
            )
            .expect("create_action_server_raw_live");

        let client = client_node
            .create_action_client_raw("cancellable")
            .expect("create_action_client_raw");
        let handle = executor.shutdown_handle().expect("shutdown handle");

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let got_fb = Arc::new(AtomicBool::new(false));
            let flag = Arc::clone(&got_fb);
            let goal = client
                .send_goal(
                    b"go",
                    None,
                    Some(Duration::from_secs(10)),
                    Some(Arc::new(move |message: &robot_bus::ActionMessage| {
                        if message.kind == ActionKind::Feedback {
                            flag.store(true, Ordering::SeqCst);
                        }
                    })),
                )
                .expect("send_goal");
            let wait_fb = Instant::now();
            while !got_fb.load(Ordering::SeqCst) && wait_fb.elapsed() < Duration::from_secs(5) {
                thread::sleep(Duration::from_millis(20));
            }
            assert!(
                got_fb.load(Ordering::SeqCst),
                "expected live FEEDBACK before cancel"
            );
            goal.cancel().expect("cancel");
            let result = goal.wait_result().expect("wait_result");
            assert_eq!(result.body, b"cancelled");
            handle.shutdown();
        });

        executor.spin().expect("spin");
        assert!(
            took_cancel.load(Ordering::SeqCst),
            "handler must observe cancel_requested"
        );
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
                .send_goal_and_wait(
                    &FibonacciGoal { order: 5 },
                    None,
                    Some(Duration::from_secs(10)),
                )
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
fn node_service_action_qos_keep_last_maps_to_hwm() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);
    let qos = robot_bus::QosProfile::keep_last(16);

    {
        let mut server = Node::with_options("qos_server", options.clone());
        let mut client_node = Node::with_options("qos_client", options);
        let executor = SingleThreadedExecutor::new();
        executor.add_node(&mut server).expect("add server");

        server
            .create_service_raw_with_qos(
                "echo",
                qos,
                Arc::new(|body| body.to_vec()),
                None,
            )
            .expect("create_service_raw_with_qos");
        assert_eq!(
            server.rpc_hwm().expect("server default rpc hwm"),
            robot_bus::HighWaterMark::RPC
        );

        let client = client_node
            .create_client_raw_with_qos("echo", qos)
            .expect("create_client_raw_with_qos");
        assert_eq!(
            client.high_water_mark().expect("client hwm"),
            robot_bus::HighWaterMark::new(16, 16)
        );

        server
            .create_action_server_raw_with_qos(
                "fib",
                qos,
                Arc::new(|_| vec![("RESULT".into(), Vec::new())]),
                None,
            )
            .expect("create_action_server_raw_with_qos");
        assert_eq!(
            server.action_hwm().expect("server default action hwm"),
            robot_bus::HighWaterMark::ACTION
        );

        let action = client_node
            .create_action_client_raw_with_qos("fib", qos)
            .expect("create_action_client_raw_with_qos");
        assert_eq!(
            action.high_water_mark().expect("action client hwm"),
            robot_bus::HighWaterMark::new(16, 16)
        );
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
