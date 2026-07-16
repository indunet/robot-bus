#![cfg(feature = "grpc")]

mod support;

use std::net::SocketAddr;
use std::time::Duration;

use robot_bus::grpc::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::grpc::pb::SubscribeRequest;
use robot_bus::grpc::{serve, GatewayConfig};
use robot_bus::Publisher;
use support::{free_port, MessageProxy};
use tokio_stream::StreamExt;
use tonic::Request;

#[tokio::test]
async fn subscribe_receives_published_payload() {
    let proxy = MessageProxy::spawn();
    let listen_port = free_port();
    let listen: SocketAddr = format!("127.0.0.1:{listen_port}").parse().unwrap();

    let config = GatewayConfig {
        listen,
        message_xpub: proxy.xpub_endpoint.clone(),
        cors_origins: Vec::new(),
    };
    tokio::spawn(async move {
        serve(config).await.expect("gateway serve");
    });

    // Wait for server bind.
    tokio::time::sleep(Duration::from_millis(150)).await;

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

    // Give ZMQ SUB time to connect and subscribe.
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
