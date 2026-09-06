//! Multiplexed WebSocket RPC smoke tests (V3: stream_id + opcode).

#![cfg(feature = "ws")]

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use robot_bus::ws_gateway::ws_frame::{
    ACTION_KIND_RESULT, Frame, RequestHeader, decode_action_data, decode_frame,
    decode_subscribe_data, encode_frame,
};
use robot_bus::{Publisher, RobotBusBroker, Subscriber};
use support::{ephemeral_robot_bus_config, lock_brokers};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use zmq::{Context as ZmqContext, SocketType};

fn start_bus() -> (support::BrokerLockGuard, RobotBusBroker) {
    let guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("start RobotBusBroker");
    (guard, broker)
}

#[tokio::test]
async fn ws_publish_reaches_zmq_subscriber() {
    let (_guard, broker) = start_bus();
    let listen = broker.api_listen();

    let sub = Subscriber::new(Some(&broker.message.xpub_bind)).expect("subscriber");
    sub.subscribe("ws.pub").expect("subscribe");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let url = format!("ws://{listen}/ws-rpc");
    let (mut ws, _) = connect_async(&url).await.expect("ws connect");

    let req = encode_frame(&Frame::Request {
        stream_id: 1,
        header: RequestHeader::Publish {
            topic: "ws.pub".into(),
        },
        body: b"from-ws".to_vec(),
    })
    .unwrap();
    ws.send(Message::Binary(req.into()))
        .await
        .expect("send request");

    let mut saw_trailer = false;
    for _ in 0..4 {
        let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .expect("timeout")
            .expect("ws closed")
            .expect("ws error");
        let Message::Binary(bin) = msg else {
            continue;
        };
        match decode_frame(&bin).expect("decode") {
            Frame::Data { .. } => panic!("publish must not send DATA ack"),
            Frame::Trailer { status, .. } => {
                assert_eq!(status, 0);
                saw_trailer = true;
                break;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(saw_trailer, "expected TRAILER");

    let (_topic, body) = sub
        .receive(Some(Duration::from_secs(2)))
        .expect("zmq receive");
    assert_eq!(body, b"from-ws");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn ws_subscribe_receives_published_payload() {
    let (_guard, broker) = start_bus();
    let listen = broker.api_listen();

    let url = format!("ws://{listen}/ws-rpc");
    let (mut ws, _) = connect_async(&url).await.expect("ws connect");

    let req = encode_frame(&Frame::Request {
        stream_id: 1,
        header: RequestHeader::Subscribe {
            topic: "ws.sub".into(),
            qos_depth: 8,
        },
        body: Vec::new(),
    })
    .unwrap();
    ws.send(Message::Binary(req.into()))
        .await
        .expect("send subscribe");

    tokio::time::sleep(Duration::from_millis(250)).await;

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    pub_.publish("ws.sub", b"hello-ws").expect("publish");

    let mut got = false;
    for _ in 0..20 {
        let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
            .await
            .expect("timeout")
            .expect("ws closed")
            .expect("ws error");
        let Message::Binary(bin) = msg else {
            continue;
        };
        if let Frame::Data { payload, .. } = decode_frame(&bin).expect("decode") {
            let (topic, body) = decode_subscribe_data(&payload).expect("subscribe data");
            assert_eq!(topic, "ws.sub");
            assert_eq!(body, b"hello-ws");
            got = true;
            break;
        }
    }
    assert!(got, "expected DATA with topic header");
    let _ = ws.close(None).await;
    broker.stop().expect("stop");
}

#[tokio::test]
async fn ws_send_goal_soft_cancel_keeps_connection_for_result() {
    let (_guard, broker) = start_bus();
    let listen = broker.api_listen();
    let backend = broker.action.backend_bind.clone();
    let cancel_seen = Arc::new(AtomicBool::new(false));
    let flag = cancel_seen.clone();
    let worker = thread::spawn(move || {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::DEALER).expect("create worker");
        socket
            .set_identity(b"ws-soft-cancel-worker")
            .expect("identity");
        socket.connect(&backend).expect("connect backend");
        socket
            .send_multipart([b"READY".as_ref(), b"act.ws_soft_cancel".as_ref()], 0)
            .expect("send ready");
        socket.set_rcvtimeo(5_000).expect("receive timeout");
        let mut client_id = Vec::new();
        let mut action = Vec::new();
        let mut goal_id = Vec::new();
        while let Ok(frames) = socket.recv_multipart(0) {
            if frames.len() == 5 && frames[3] == b"GOAL" {
                client_id = frames[0].clone();
                action = frames[1].clone();
                goal_id = frames[2].clone();
                continue;
            }
            if frames.len() == 5 && frames[3] == b"CANCEL" {
                flag.store(true, Ordering::Release);
                let _ = socket.send_multipart(
                    [
                        client_id.as_slice(),
                        action.as_slice(),
                        goal_id.as_slice(),
                        b"RESULT".as_ref(),
                        b"cancelled-ok".as_ref(),
                    ],
                    0,
                );
                break;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let url = format!("ws://{listen}/ws-rpc");
    let (mut ws, _) = connect_async(&url).await.expect("ws connect");
    ws.send(Message::Binary(
        encode_frame(&Frame::Request {
            stream_id: 1,
            header: RequestHeader::SendGoal {
                action_name: "act.ws_soft_cancel".into(),
                goal_id: "soft-goal".into(),
                timeout_ms: 10_000,
            },
            body: b"wait".to_vec(),
        })
        .unwrap()
        .into(),
    ))
    .await
    .expect("send goal");

    tokio::time::sleep(Duration::from_millis(150)).await;
    ws.send(Message::Binary(
        encode_frame(&Frame::Cancel { stream_id: 1 })
            .unwrap()
            .into(),
    ))
    .await
    .expect("send CANCEL");

    let mut got_result = false;
    for _ in 0..40 {
        let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
            .await
            .expect("timeout")
            .expect("ws closed")
            .expect("ws error");
        let Message::Binary(bin) = msg else {
            continue;
        };
        match decode_frame(&bin).expect("decode") {
            Frame::Data { payload, .. } => {
                let (kind, body) = decode_action_data(&payload).expect("event");
                if kind == ACTION_KIND_RESULT {
                    assert_eq!(body, b"cancelled-ok");
                    got_result = true;
                }
            }
            Frame::Trailer { status, .. } => {
                assert_eq!(status, 0);
                break;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(
        cancel_seen.load(Ordering::Acquire),
        "worker should observe CANCEL"
    );
    assert!(got_result, "soft cancel should still deliver RESULT");
    worker.join().expect("join worker");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn ws_send_goal_disconnect_still_submits_cancel() {
    let (_guard, broker) = start_bus();
    let listen = broker.api_listen();
    let backend = broker.action.backend_bind.clone();
    let cancel_seen = Arc::new(AtomicBool::new(false));
    let flag = cancel_seen.clone();
    let worker = thread::spawn(move || {
        let context = ZmqContext::new();
        let socket = context.socket(SocketType::DEALER).expect("create worker");
        socket
            .set_identity(b"ws-disconnect-cancel-worker")
            .expect("identity");
        socket.connect(&backend).expect("connect backend");
        socket
            .send_multipart([b"READY".as_ref(), b"act.ws_disconnect_cancel".as_ref()], 0)
            .expect("send ready");
        socket.set_rcvtimeo(5_000).expect("receive timeout");
        while let Ok(frames) = socket.recv_multipart(0) {
            if frames.len() >= 5 && frames[3] == b"CANCEL" {
                flag.store(true, Ordering::Release);
                break;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let url = format!("ws://{listen}/ws-rpc");
    let (mut ws, _) = connect_async(&url).await.expect("ws connect");
    ws.send(Message::Binary(
        encode_frame(&Frame::Request {
            stream_id: 1,
            header: RequestHeader::SendGoal {
                action_name: "act.ws_disconnect_cancel".into(),
                goal_id: "disconnect-goal".into(),
                timeout_ms: 10_000,
            },
            body: b"wait".to_vec(),
        })
        .unwrap()
        .into(),
    ))
    .await
    .expect("send goal");

    tokio::time::sleep(Duration::from_millis(150)).await;
    let _ = ws.close(None).await;

    tokio::time::timeout(Duration::from_secs(3), async {
        while !cancel_seen.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("worker did not observe cancel after disconnect");

    worker.join().expect("join worker");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn ws_multiplex_two_streams_on_one_connection() {
    let (_guard, broker) = start_bus();
    let listen = broker.api_listen();
    let url = format!("ws://{listen}/ws-rpc");
    let (mut ws, _) = connect_async(&url).await.expect("ws connect");

    ws.send(Message::Binary(
        encode_frame(&Frame::Request {
            stream_id: 1,
            header: RequestHeader::Subscribe {
                topic: "mux.a".into(),
                qos_depth: 0,
            },
            body: Vec::new(),
        })
        .unwrap()
        .into(),
    ))
    .await
    .expect("subscribe");

    tokio::time::sleep(Duration::from_millis(200)).await;

    ws.send(Message::Binary(
        encode_frame(&Frame::Request {
            stream_id: 3,
            header: RequestHeader::Publish {
                topic: "mux.a".into(),
            },
            body: b"mux-hi".to_vec(),
        })
        .unwrap()
        .into(),
    ))
    .await
    .expect("publish");

    let mut saw_pub_trailer = false;
    let mut saw_sub_data = false;
    for _ in 0..20 {
        let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .expect("timeout")
            .expect("ws closed")
            .expect("ws error");
        let Message::Binary(bin) = msg else {
            continue;
        };
        match decode_frame(&bin).expect("decode") {
            Frame::Data { stream_id, payload } if stream_id == 1 => {
                let (_topic, body) = decode_subscribe_data(&payload).expect("topic");
                assert_eq!(body, b"mux-hi");
                saw_sub_data = true;
            }
            Frame::Trailer {
                stream_id, status, ..
            } if stream_id == 3 => {
                assert_eq!(status, 0);
                saw_pub_trailer = true;
            }
            _ => {}
        }
        if saw_pub_trailer && saw_sub_data {
            break;
        }
    }
    assert!(saw_pub_trailer && saw_sub_data, "multiplex pub+sub failed");
    broker.stop().expect("stop");
}

#[tokio::test]
async fn ws_gateway_echoes_ping_with_pong() {
    let (_guard, broker) = start_bus();
    let listen = broker.api_listen();
    let url = format!("ws://{listen}/ws-rpc");
    let (mut ws, _) = connect_async(&url).await.expect("ws connect");

    let req = encode_frame(&Frame::Ping { stream_id: 0 }).unwrap();
    ws.send(Message::Binary(req.into()))
        .await
        .expect("send ping");

    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timeout")
        .expect("ws closed")
        .expect("ws error");
    let Message::Binary(bin) = msg else {
        panic!("expected binary pong, got {msg:?}");
    };
    match decode_frame(&bin).expect("decode") {
        Frame::Pong { stream_id } => assert_eq!(stream_id, 0),
        other => panic!("expected PONG, got {other:?}"),
    }
    broker.stop().expect("stop");
}
