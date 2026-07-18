#![cfg(feature = "grpc")]

mod support;

use std::sync::Arc;
use std::time::Duration;

use robot_bus::grpc::pb::action_gateway_client::ActionGatewayClient;
use robot_bus::grpc::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::grpc::pb::service_gateway_client::ServiceGatewayClient;
use robot_bus::grpc::pb::{
    action_client_message, ActionClientMessage, ActionKind, CancelCommand, GoalCommand,
    ServiceCallRequest, SubscribeRequest,
};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{Publisher, RobotBusBroker};
use support::{ephemeral_robot_bus_config, lock_brokers};
use tokio_stream::StreamExt;
use tonic::Code;
use tonic::Request;

fn start_bus() -> (std::sync::MutexGuard<'static, ()>, RobotBusBroker) {
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
async fn action_run_streams_feedback_then_result() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|body| {
            vec![
                ("FEEDBACK".into(), b"step-1".to_vec()),
                ("FEEDBACK".into(), b"step-2".to_vec()),
                ("RESULT".into(), [b"done:", body].concat()),
            ]
        });
    let worker = WorkerThread::spawn_action("act.grpc_demo", handler, &broker.action.backend_bind)
        .expect("worker");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = ActionGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect");
    let outbound = tokio_stream::iter(vec![ActionClientMessage {
        msg: Some(action_client_message::Msg::Goal(GoalCommand {
            action_name: "act.grpc_demo".into(),
            goal: b"fly".to_vec(),
            goal_id: String::new(),
            timeout_ms: 10_000,
        })),
    }]);
    let mut stream = client
        .run(Request::new(outbound))
        .await
        .expect("run")
        .into_inner();

    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        events.push(item.expect("stream status"));
    }

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].kind, ActionKind::Feedback as i32);
    assert_eq!(events[0].body, b"step-1");
    assert_eq!(events[1].kind, ActionKind::Feedback as i32);
    assert_eq!(events[1].body, b"step-2");
    assert_eq!(events[2].kind, ActionKind::Result as i32);
    assert_eq!(events[2].body, b"done:fly");

    worker.stop();
    broker.stop().expect("stop");
}

#[tokio::test]
async fn action_run_cancel_unknown_goal_returns_not_found() {
    let (_guard, broker) = start_bus();
    let listen = broker.grpc_listen();

    let mut client = ActionGatewayClient::connect(format!("http://{listen}"))
        .await
        .expect("connect");
    let outbound = tokio_stream::iter(vec![ActionClientMessage {
        msg: Some(action_client_message::Msg::Cancel(CancelCommand {
            action_name: "act.missing".into(),
            goal_id: "no-such-goal".into(),
            body: Vec::new(),
        })),
    }]);
    let mut stream = client
        .run(Request::new(outbound))
        .await
        .expect("run")
        .into_inner();

    let err = stream
        .next()
        .await
        .expect("expected stream item")
        .expect_err("expected not found");
    assert_eq!(err.code(), Code::NotFound);
    broker.stop().expect("stop");
}
