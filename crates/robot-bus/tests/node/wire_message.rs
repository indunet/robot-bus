//! A codec that is not protobuf can use the typed Node APIs.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::common::{ephemeral_robot_bus_config, lock_brokers};
use robot_bus::typed::{Service, WireError, WireMessage};
use robot_bus::{Node, NodeOptions, RobotBusBroker};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TextMsg(String);

impl WireMessage for TextMsg {
    fn to_wire(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, WireError> {
        let text = std::str::from_utf8(bytes).map_err(|err| WireError::new(err.to_string()))?;
        Ok(Self(text.to_string()))
    }

    fn wire_type_name() -> Option<String> {
        Some("text".into())
    }
}

struct EchoText;

impl Service for EchoText {
    type Request = TextMsg;
    type Response = TextMsg;
}

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
fn custom_codec_round_trips_a_topic_and_a_service() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let options = node_options_from_broker(&broker);

    let mut sub_node = Node::with_options("text_sub", options.clone());
    let mut pub_node = Node::with_options("text_pub", options.clone());
    let got = Arc::new(std::sync::Mutex::new(None));
    let slot = Arc::clone(&got);
    sub_node
        .create_subscription::<TextMsg, _>(
            "/text",
            move |msg| *slot.lock().unwrap() = Some(msg.0),
            None,
        )
        .expect("subscribe");
    let publisher = pub_node
        .create_publisher::<TextMsg>("/text")
        .expect("publish");
    let sub_stop = sub_node.shutdown_handle().expect("sub stop");
    thread::spawn(move || {
        sub_node.spin().expect("spin sub");
    });
    thread::sleep(Duration::from_millis(200));
    publisher
        .publish(&TextMsg("hello".into()))
        .expect("publish text");

    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        if got.lock().unwrap().as_deref() == Some("hello") {
            break;
        }
        if std::time::Instant::now() >= deadline {
            panic!("timed out waiting for text topic: {:?}", got.lock().unwrap());
        }
        thread::sleep(Duration::from_millis(20));
    }
    sub_stop.shutdown();

    let mut server = Node::with_options("text_server", options.clone());
    let mut client_node = Node::with_options("text_client", options);
    server
        .create_service::<EchoText, _>("echo.text", |req| TextMsg(format!("echo:{}", req.0)), None)
        .expect("create_service");
    let client = client_node
        .create_client::<EchoText>("echo.text")
        .expect("create_client");
    let handle = server.shutdown_handle().expect("shutdown handle");
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        let reply = client
            .call(&TextMsg("ping".into()), Some(Duration::from_secs(5)))
            .expect("call");
        assert_eq!(reply.0, "echo:ping");
        handle.shutdown();
    });
    server.spin().expect("spin");
    broker.stop().expect("stop broker");
}
