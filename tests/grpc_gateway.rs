#![cfg(feature = "grpc")]

mod support;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::grpc::pb::action_gateway_client::ActionGatewayClient;
use robot_bus::grpc::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::grpc::pb::service_gateway_client::ServiceGatewayClient;
use robot_bus::grpc::pb::{
    action_client_message, ActionClientMessage, ActionKind, CancelCommand, GoalCommand,
    ServiceCallRequest, SubscribeRequest,
};
use robot_bus::grpc::{serve, GatewayConfig};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{ActionBusBroker, Publisher, ServiceBusBroker};
use support::{free_port, MessageProxy};
use tokio_stream::StreamExt;
use tonic::Code;
use tonic::Request;

fn listen_addr() -> SocketAddr {
    format!("127.0.0.1:{}", free_port()).parse().unwrap()
}

async fn spawn_gateway(config: GatewayConfig) -> SocketAddr {
    let listen = config.listen;
    tokio::spawn(async move {
        serve(config).await.expect("gateway serve");
    });
    tokio::time::sleep(Duration::from_millis(150)).await;
    listen
}

#[tokio::test]
async fn subscribe_receives_published_payload() {
    let proxy = MessageProxy::spawn();
    let listen = listen_addr();

    let config = GatewayConfig {
        listen,
        message_xpub: proxy.xpub_endpoint.clone(),
        service_frontend: "tcp://127.0.0.1:1".into(),
        action_frontend: "tcp://127.0.0.1:1".into(),
        cors_origins: Vec::new(),
    };
    let listen = spawn_gateway(config).await;

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

    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    pub_.publish("grpc.test", b"hello-grpc").expect("publish");

    let msg = tokio::time::timeout(Duration::from_secs(3), stream.next())
        .await
        .expect("timeout waiting for stream item")
        .expect("stream ended")
        .expect("stream status");

    assert_eq!(msg.topic, "grpc.test");
    assert_eq!(msg.payload, b"hello-grpc");
}

#[tokio::test]
async fn service_call_echoes_payload() {
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| [b"echo:", body].concat());
    let worker = WorkerThread::spawn_service("svc.grpc_echo", handler, &broker.backend_bind)
        .expect("worker");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let listen = listen_addr();
    let config = GatewayConfig {
        listen,
        message_xpub: "tcp://127.0.0.1:1".into(),
        service_frontend: broker.frontend_bind.clone(),
        action_frontend: "tcp://127.0.0.1:1".into(),
        cors_origins: Vec::new(),
    };
    let listen = spawn_gateway(config).await;

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
    let _ = broker.stop();
}

#[tokio::test]
async fn action_run_streams_feedback_then_result() {
    let broker = ActionBusBroker::start(ActionBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        bind_all_transports: false,
        ..ActionBusConfig::default()
    })
    .expect("start action bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|_client_id, _goal_id, body| {
            vec![
                ("FEEDBACK".into(), b"step-1".to_vec()),
                ("FEEDBACK".into(), b"step-2".to_vec()),
                ("RESULT".into(), [b"done:", body].concat()),
            ]
        });
    let worker = WorkerThread::spawn_action("act.grpc_demo", handler, &broker.backend_bind)
        .expect("worker");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let listen = listen_addr();
    let config = GatewayConfig {
        listen,
        message_xpub: "tcp://127.0.0.1:1".into(),
        service_frontend: "tcp://127.0.0.1:1".into(),
        action_frontend: broker.frontend_bind.clone(),
        cors_origins: Vec::new(),
    };
    let listen = spawn_gateway(config).await;

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
    let _ = broker.stop();
}

#[tokio::test]
async fn action_run_cancel_unknown_goal_returns_not_found() {
    let broker = ActionBusBroker::start(ActionBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        bind_all_transports: false,
        ..ActionBusConfig::default()
    })
    .expect("start action bus");

    let listen = listen_addr();
    let config = GatewayConfig {
        listen,
        message_xpub: "tcp://127.0.0.1:1".into(),
        service_frontend: "tcp://127.0.0.1:1".into(),
        action_frontend: broker.frontend_bind.clone(),
        cors_origins: Vec::new(),
    };
    let listen = spawn_gateway(config).await;

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
    let _ = broker.stop();
}
