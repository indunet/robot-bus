#![cfg(feature = "ws")]

mod support;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use robot_bus::worker_thread::WorkerThread;
use robot_bus::{Node, Publisher, QosProfile, RobotBusBroker};
use support::{ephemeral_robot_bus_config, lock_brokers};

fn start_bus() -> (support::BrokerLockGuard, RobotBusBroker) {
    let guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("start RobotBusBroker");
    (guard, broker)
}

fn ws_url(broker: &RobotBusBroker) -> String {
    broker.api_url()
}

#[test]
fn ws_node_subscribe_receives_published_payload() {
    let (_guard, broker) = start_bus();
    let url = ws_url(&broker);

    let got = Arc::new(Mutex::new(None::<Vec<u8>>));
    let got_cb = Arc::clone(&got);
    let mut node = Node::ws_at("ws-sub", &url);
    node.create_subscription_raw(
        "ws.node.topic",
        Arc::new(move |payload| {
            *got_cb.lock().unwrap() = Some(payload.to_vec());
        }),
        None,
    )
    .expect("subscribe");

    // Allow the async subscribe stream to connect.
    thread::sleep(Duration::from_millis(300));

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    pub_.publish("ws.node.topic", b"hello-ws-node")
        .expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while got.lock().unwrap().is_none() && std::time::Instant::now() < deadline {
        node.spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
    }

    let payload = got.lock().unwrap().clone().expect("callback fired");
    assert_eq!(payload, b"hello-ws-node");
    broker.stop().expect("stop");
}

#[test]
fn ws_node_subscribe_with_qos_receives_published_payload() {
    let (_guard, broker) = start_bus();
    let url = ws_url(&broker);

    let got = Arc::new(Mutex::new(None::<Vec<u8>>));
    let got_cb = Arc::clone(&got);
    let mut node = Node::ws_at("ws-sub-qos", &url);
    node.create_subscription_raw_with_qos(
        "ws.node.qos",
        QosProfile::keep_last(4),
        Arc::new(move |payload| {
            *got_cb.lock().unwrap() = Some(payload.to_vec());
        }),
        None,
    )
    .expect("subscribe");

    thread::sleep(Duration::from_millis(300));

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    pub_.publish("ws.node.qos", b"qos-hi").expect("publish");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while got.lock().unwrap().is_none() && std::time::Instant::now() < deadline {
        node.spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
    }

    assert_eq!(
        got.lock().unwrap().clone().expect("callback fired"),
        b"qos-hi"
    );
    broker.stop().expect("stop");
}

#[test]
fn ws_node_service_call_echoes_payload() {
    let (_guard, broker) = start_bus();
    let url = ws_url(&broker);

    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"echo:", body].concat());
    let worker =
        WorkerThread::spawn_service("svc.ws_node_echo", handler, &broker.service.backend_bind)
            .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let mut node = Node::ws_at("ws-client", &url);
    let client = node
        .create_client_raw("svc.ws_node_echo")
        .expect("create_client");
    let resp = client
        .call(b"ping", Some(Duration::from_secs(3)))
        .expect("call");
    assert_eq!(resp, b"echo:ping");

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn ws_node_action_client_streams_feedback_then_result() {
    let (_guard, broker) = start_bus();
    let url = ws_url(&broker);

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> = Arc::new(|body| {
        vec![
            ("FEEDBACK".into(), b"step-1".to_vec()),
            ("FEEDBACK".into(), b"step-2".to_vec()),
            ("RESULT".into(), [b"done:", body].concat()),
        ]
    });
    let worker =
        WorkerThread::spawn_action("act.ws_node_demo", handler, &broker.action.backend_bind)
            .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let mut node = Node::ws_at("grpc-action", &url);
    let client = node
        .create_action_client_raw("act.ws_node_demo")
        .expect("create_action_client");
    let feedbacks = Arc::new(Mutex::new(Vec::new()));
    let callback_feedbacks = Arc::clone(&feedbacks);
    let goal = client
        .send_goal(
            b"fly",
            None,
            Some(Duration::from_secs(5)),
            Some(Arc::new(move |message: &robot_bus::ActionMessage| {
                callback_feedbacks
                    .lock()
                    .expect("feedback mutex")
                    .push(message.body.clone());
            })),
        )
        .expect("send_goal");
    assert_eq!(goal.action_name(), "act.ws_node_demo");
    assert!(!goal.goal_id().is_empty());
    let messages = goal.collect().expect("collect");

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].kind, robot_bus::ActionKind::Feedback);
    assert_eq!(messages[0].body, b"step-1");
    assert_eq!(messages[1].kind, robot_bus::ActionKind::Feedback);
    assert_eq!(messages[1].body, b"step-2");
    assert_eq!(messages[2].kind, robot_bus::ActionKind::Result);
    assert_eq!(messages[2].body, b"done:fly");
    assert_eq!(
        *feedbacks.lock().expect("feedback mutex"),
        vec![b"step-1".to_vec(), b"step-2".to_vec()]
    );

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn ws_node_publish_reaches_zmq_subscriber() {
    let (_guard, broker) = start_bus();
    let url = ws_url(&broker);

    let sub = robot_bus::Subscriber::new(Some(&broker.message.xpub_bind)).expect("subscriber");
    sub.subscribe("ws.node.pub").expect("subscribe");
    thread::sleep(Duration::from_millis(200));

    let mut node = Node::ws_at("grpc-pub", &url);
    let pub_ = node
        .create_publisher_raw("ws.node.pub")
        .expect("create_publisher");
    pub_.publish(b"hello-from-grpc-node").expect("publish");

    let (topic, payload) = sub.receive(Some(Duration::from_secs(3))).expect("receive");
    assert_eq!(topic, "ws.node.pub");
    assert_eq!(payload, b"hello-from-grpc-node");
    broker.stop().expect("stop");
}

#[test]
fn ws_node_rejects_service_and_action_server() {
    let mut node = Node::ws("grpc-only");
    assert!(
        node.create_service_raw("/svc", Arc::new(|_| Vec::new()), None)
            .is_err()
    );
    assert!(
        node.create_action_server_raw(
            "/act",
            Arc::new(|_| vec![("RESULT".into(), Vec::new())]),
            None
        )
        .is_err()
    );
}

#[test]
fn ws_node_resubscribes_after_broker_restart() {
    let _guard = lock_brokers();
    let api_port = support::free_port();
    let mut config = ephemeral_robot_bus_config();
    config.ws.listen = format!("127.0.0.1:{api_port}").parse().unwrap();
    let broker = RobotBusBroker::start(config.clone()).expect("start RobotBusBroker");
    let url = ws_url(&broker);

    let got = Arc::new(Mutex::new(None::<Vec<u8>>));
    let got_cb = Arc::clone(&got);
    let mut node = Node::ws_at("ws-re", &url);
    node.create_subscription_raw(
        "ws.reconnect.topic",
        Arc::new(move |payload| {
            *got_cb.lock().unwrap() = Some(payload.to_vec());
        }),
        None,
    )
    .expect("subscribe");
    thread::sleep(Duration::from_millis(300));

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    pub_.publish("ws.reconnect.topic", b"before-restart")
        .expect("publish");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while got.lock().unwrap().is_none() && std::time::Instant::now() < deadline {
        node.spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
    }
    assert_eq!(
        got.lock().unwrap().clone().as_deref(),
        Some(&b"before-restart"[..])
    );
    broker.stop().expect("stop");

    *got.lock().unwrap() = None;
    thread::sleep(Duration::from_millis(600));
    let broker = RobotBusBroker::start(config).expect("restart broker");
    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    while got.lock().unwrap().is_none() && std::time::Instant::now() < deadline {
        let _ = pub_.publish("ws.reconnect.topic", b"after-restart");
        node.spin_once(Some(Duration::from_millis(50)))
            .expect("spin_once");
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        got.lock().unwrap().clone().as_deref(),
        Some(&b"after-restart"[..]),
        "subscription should resume after broker restart"
    );
    broker.stop().expect("stop");
}

#[test]
fn ws_node_connection_state_follows_socket() {
    let _guard = lock_brokers();
    let api_port = support::free_port();
    let mut config = ephemeral_robot_bus_config();
    config.ws.listen = format!("127.0.0.1:{api_port}").parse().unwrap();
    let broker = RobotBusBroker::start(config).expect("start RobotBusBroker");
    let url = ws_url(&broker);

    let node = Node::ws_at("ws-state", &url);
    assert!(
        node.wait_for_broker(Some(Duration::from_secs(5))),
        "expected connected, state={}",
        node.connection_state()
    );
    assert_eq!(
        node.connection_state(),
        robot_bus::ConnectionState::Connected
    );

    broker.stop().expect("stop");
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    while node.connection_state() != robot_bus::ConnectionState::Reconnecting
        && std::time::Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        node.connection_state(),
        robot_bus::ConnectionState::Reconnecting,
        "socket drop should mark reconnecting"
    );
}
