#![cfg(feature = "grpc")]

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use robot_bus::grpc::pb::action_gateway_client::ActionGatewayClient;
use robot_bus::grpc::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::grpc::pb::service_gateway_client::ServiceGatewayClient;
use robot_bus::grpc::pb::{
    ActionKind, GoalCommand, ServiceCallRequest, SubscribeRequest, TopicMessage,
};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{Publisher, RobotBusBroker};
use support::{ephemeral_robot_bus_config, lock_brokers};
use tokio_stream::StreamExt;
use tonic::Code;
use tonic::Request;
use zmq::{Context as ZmqContext, SocketType};

fn start_bus() -> (support::BrokerLockGuard, RobotBusBroker) {
    let guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("start RobotBusBroker");
    (guard, broker)
}

#[tokio::test]
async fn subscribe_receives_published_payload() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let mut client = MessageGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect gateway");

    let mut stream = client
        .subscribe(Request::new(SubscribeRequest {
            topic: "grpc.test".into(),
        }))
        .await
        .expect("subscribe")
        .into_inner();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    pub_.publish("grpc.test", b"hello-grpc").expect("publish");

    let msg = tokio::time::timeout(Duration::from_secs(3), stream.next())
        .await
        .expect("timeout waiting for stream item")
        .expect("stream ended")
        .expect("stream status");

    assert_eq!(msg.topic, "grpc.test");
    assert_eq!(msg.payload, b"hello-grpc");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn publish_reaches_zmq_subscriber() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let sub = robot_bus::Subscriber::new(Some(&broker.message.xpub_bind)).expect("subscriber");
    sub.subscribe("grpc.pub").expect("subscribe");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut client = MessageGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect gateway");
    client
        .publish(Request::new(TopicMessage {
            topic: "grpc.pub".into(),
            payload: b"from-grpc".to_vec(),
        }))
        .await
        .expect("publish");

    let (topic, payload) = tokio::task::spawn_blocking(move || {
        sub.receive(Some(Duration::from_secs(3)))
            .expect("receive published message")
    })
    .await
    .expect("join");

    assert_eq!(topic, "grpc.pub");
    assert_eq!(payload, b"from-grpc");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn publish_empty_topic_is_invalid_argument() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let mut client = MessageGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect gateway");
    let err = client
        .publish(Request::new(TopicMessage {
            topic: String::new(),
            payload: b"x".to_vec(),
        }))
        .await
        .expect_err("empty topic");
    assert_eq!(err.code(), Code::InvalidArgument);
    broker.stop().expect("stop");
}

#[tokio::test]
async fn service_call_echoes_payload() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"echo:", body].concat());
    let worker =
        WorkerThread::spawn_service("svc.grpc_echo", handler, &broker.service.backend_bind)
            .expect("worker");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = ServiceGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect");
    let resp = client
        .call(Request::new(ServiceCallRequest {
            service_name: "svc.grpc_echo".into(),
            request: b"ping".to_vec(),
            request_id: String::new(),
            timeout_ms: 5_000,
        }))
        .await
        .expect("call")
        .into_inner();

    assert_eq!(resp.response, b"echo:ping");
    worker.stop();
    broker.stop().expect("stop");
}

#[tokio::test]
async fn action_send_goal_streams_feedback_then_result() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> = Arc::new(|body| {
        vec![
            ("FEEDBACK".into(), b"step-1".to_vec()),
            ("FEEDBACK".into(), b"step-2".to_vec()),
            ("RESULT".into(), [b"done:", body].concat()),
        ]
    });
    let worker =
        WorkerThread::spawn_action("act.grpc_web_demo", handler, &broker.action.backend_bind)
            .expect("worker");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = ActionGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect");
    let mut stream = client
        .send_goal(Request::new(GoalCommand {
            action_name: "act.grpc_web_demo".into(),
            goal: b"fly".to_vec(),
            goal_id: "web-goal".into(),
            timeout_ms: 10_000,
        }))
        .await
        .expect("send goal")
        .into_inner();

    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        events.push(item.expect("stream status"));
    }

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].goal_id, "web-goal");
    assert_eq!(events[0].kind, ActionKind::Feedback as i32);
    assert_eq!(events[1].kind, ActionKind::Feedback as i32);
    assert_eq!(events[2].kind, ActionKind::Result as i32);
    assert_eq!(events[2].body, b"done:fly");

    worker.stop();
    broker.stop().expect("stop");
}

#[tokio::test]
async fn action_send_goal_disconnect_submits_cancel() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();
    let cancel_seen = Arc::new(AtomicBool::new(false));
    let worker_cancel_seen = cancel_seen.clone();
    let backend = broker.action.backend_bind.clone();
    let worker = thread::spawn(move || {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::DEALER).expect("create worker");
        socket
            .set_identity(b"grpc-ws-cancel-worker")
            .expect("identity");
        socket.connect(&backend).expect("connect backend");
        socket
            .send_multipart([b"READY".as_ref(), b"act.grpc_web_cancel".as_ref()], 0)
            .expect("send ready");
        socket.set_rcvtimeo(3_000).expect("receive timeout");
        while let Ok(frames) = socket.recv_multipart(0) {
            if frames.len() == 5 && frames[3] == b"CANCEL" {
                worker_cancel_seen.store(true, Ordering::Release);
                break;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = ActionGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect");
    let stream = client
        .send_goal(Request::new(GoalCommand {
            action_name: "act.grpc_web_cancel".into(),
            goal: b"wait".to_vec(),
            goal_id: "disconnect-goal".into(),
            timeout_ms: 10_000,
        }))
        .await
        .expect("send goal")
        .into_inner();
    drop(stream);

    tokio::time::timeout(Duration::from_secs(3), async {
        while !cancel_seen.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("worker did not observe cancel");

    worker.join().expect("join worker");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn action_send_goal_timeout_submits_cancel() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();
    let backend = broker.action.backend_bind.clone();
    let worker = thread::spawn(move || {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::DEALER).expect("create worker");
        socket
            .set_identity(b"grpc-ws-timeout-worker")
            .expect("identity");
        socket.connect(&backend).expect("connect backend");
        socket
            .send_multipart([b"READY".as_ref(), b"act.grpc_web_timeout".as_ref()], 0)
            .expect("send ready");
        socket.set_rcvtimeo(3_000).expect("receive timeout");
        while let Ok(frames) = socket.recv_multipart(0) {
            if frames.len() == 5 && frames[3] == b"CANCEL" {
                return true;
            }
        }
        false
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = ActionGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect");
    let mut stream = client
        .send_goal(Request::new(GoalCommand {
            action_name: "act.grpc_web_timeout".into(),
            goal: b"wait".to_vec(),
            goal_id: "timeout-goal".into(),
            timeout_ms: 100,
        }))
        .await
        .expect("send goal")
        .into_inner();
    let error = stream
        .next()
        .await
        .expect("expected timeout status")
        .expect_err("expected deadline exceeded");
    assert_eq!(error.code(), Code::DeadlineExceeded);
    assert!(
        worker.join().expect("join worker"),
        "worker did not observe cancel"
    );

    broker.stop().expect("stop");
}
