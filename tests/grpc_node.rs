#![cfg(feature = "grpc")]

mod support;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use robot_bus::worker_thread::WorkerThread;
use robot_bus::{Node, Publisher, RobotBusBroker};
use support::{ephemeral_robot_bus_config, lock_brokers};

fn start_bus() -> (std::sync::MutexGuard<'static, ()>, RobotBusBroker) {
    let guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("start RobotBusBroker");
    (guard, broker)
}

fn grpc_url(broker: &RobotBusBroker) -> String {
    format!("http://{}", broker.grpc_listen())
}

#[test]
fn grpc_node_subscribe_receives_published_payload() {
    let (_guard, broker) = start_bus();
    let url = grpc_url(&broker);

    let got = Arc::new(Mutex::new(None::<(String, Vec<u8>)>));
    let got_cb = Arc::clone(&got);
    let mut node = Node::grpc_at("grpc-sub", &url);
    node.create_subscription_raw(
        "grpc.node.topic",
        Arc::new(move |topic, payload| {
            *got_cb.lock().unwrap() = Some((topic.to_string(), payload.to_vec()));
        }),
        None,
    )
    .expect("subscribe");

    // Allow the async subscribe stream to connect.
    thread::sleep(Duration::from_millis(300));

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    pub_
        .publish("grpc.node.topic", b"hello-grpc-node")
        .expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while got.lock().unwrap().is_none() && std::time::Instant::now() < deadline {
        node.spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
    }

    let (topic, payload) = got.lock().unwrap().clone().expect("callback fired");
    assert_eq!(topic, "grpc.node.topic");
    assert_eq!(payload, b"hello-grpc-node");
    broker.stop().expect("stop");
}

#[test]
fn grpc_node_service_call_echoes_payload() {
    let (_guard, broker) = start_bus();
    let url = grpc_url(&broker);

    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"echo:", body].concat());
    let worker =
        WorkerThread::spawn_service("svc.grpc_node_echo", handler, &broker.service.backend_bind)
            .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let mut node = Node::grpc_at("grpc-client", &url);
    let client = node
        .create_client_raw("svc.grpc_node_echo")
        .expect("create_client");
    let resp = client
        .call(b"ping", Some(Duration::from_secs(3)))
        .expect("call");
    assert_eq!(resp, b"echo:ping");

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn grpc_node_action_client_streams_feedback_then_result() {
    let (_guard, broker) = start_bus();
    let url = grpc_url(&broker);

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|body| {
            vec![
                ("FEEDBACK".into(), b"step-1".to_vec()),
                ("FEEDBACK".into(), b"step-2".to_vec()),
                ("RESULT".into(), [b"done:", body].concat()),
            ]
        });
    let worker =
        WorkerThread::spawn_action("act.grpc_node_demo", handler, &broker.action.backend_bind)
            .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let mut node = Node::grpc_at("grpc-action", &url);
    let client = node
        .create_action_client_raw("act.grpc_node_demo")
        .expect("create_action_client");
    let messages = client
        .send_goal(b"fly", None, Some(Duration::from_secs(5)))
        .expect("send_goal");

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].kind, robot_bus::ActionKind::Feedback);
    assert_eq!(messages[0].body, b"step-1");
    assert_eq!(messages[1].kind, robot_bus::ActionKind::Feedback);
    assert_eq!(messages[1].body, b"step-2");
    assert_eq!(messages[2].kind, robot_bus::ActionKind::Result);
    assert_eq!(messages[2].body, b"done:fly");

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn grpc_node_rejects_publisher_and_service_server() {
    let mut node = Node::grpc("grpc-only");
    assert!(node.create_publisher_raw("/t").is_err());
    assert!(node
        .create_service_raw("/svc", Arc::new(|_| Vec::new()), None)
        .is_err());
    assert!(node
        .create_action_server_raw("/act", Arc::new(|_| vec![("RESULT".into(), Vec::new())]), None)
        .is_err());
}
