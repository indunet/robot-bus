//! Multiplexed WebSocket RPC server (V3: one connection, many streams).

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use super::action::{GoalSpec, SendGoalSession, WsActionHandler};
use super::message::WsMessageService;
use super::rpc_status::{Code, RpcStatus};
use super::service::WsServiceHandler;
use super::ws_frame::{
    ACTION_KIND_RESULT, Frame, Opcode, RequestHeader, decode_frame, encode_action_data,
    encode_frame, encode_subscribe_data,
};

#[derive(Clone)]
pub struct WsServerState {
    pub message: WsMessageService,
    pub service: WsServiceHandler,
    pub action: WsActionHandler,
}

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

enum Outbound {
    Frame(Frame),
    Bytes(Vec<u8>),
    Subscription {
        stream_id: u32,
        queue: Arc<super::subscription_queue::SubscriptionQueue>,
        sent: tokio::sync::oneshot::Sender<()>,
    },
}

enum StreamCmd {
    Cancel,
}

struct LiveStream {
    cmd_tx: mpsc::Sender<StreamCmd>,
}

async fn handle_socket(socket: WebSocket, state: Arc<WsServerState>) {
    let (mut sink, mut stream) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<Outbound>(256);
    let mut live: HashMap<u32, LiveStream> = HashMap::new();

    loop {
        tokio::select! {
            biased;
            outbound = out_rx.recv() => {
                match outbound {
                    Some(Outbound::Frame(frame)) => {
                        if send_frame(&mut sink, &frame).await.is_err() {
                            break;
                        }
                    }
                    Some(Outbound::Bytes(bytes)) => {
                        if sink.send(Message::Binary(bytes.into())).await.is_err() {
                            break;
                        }
                    }
                    Some(Outbound::Subscription { stream_id, queue, sent }) => {
                        if let Some(msg) = queue.pop() {
                            let Ok(bytes) = encode_subscribe_data(stream_id, &msg.topic, &msg.payload) else { break; };
                            if sink.send(Message::Binary(bytes.into())).await.is_err() { break; }
                        }
                        let _ = sent.send(());
                    }
                    None => break,
                }
            }
            client_msg = stream.next() => {
                match client_msg {
                    Some(Ok(Message::Binary(bin))) => {
                        let frame = match decode_frame(&bin) {
                            Ok(f) => f,
                            Err(err) => {
                                let _ = out_tx.send(Outbound::Frame(Frame::Trailer {
                                    stream_id: 0,
                                    status: Code::InvalidArgument as u32,
                                    message: err.to_string(),
                                })).await;
                                continue;
                            }
                        };
                        match frame {
                            Frame::Request { stream_id, header, body } => {
                                if live.contains_key(&stream_id) {
                                    let _ = out_tx.send(Outbound::Frame(Frame::Trailer {
                                        stream_id,
                                        status: Code::InvalidArgument as u32,
                                        message: format!("stream_id {stream_id} already in use"),
                                    })).await;
                                    continue;
                                }
                                let (cmd_tx, cmd_rx) = mpsc::channel::<StreamCmd>(4);
                                live.insert(stream_id, LiveStream { cmd_tx });
                                let out_tx = out_tx.clone();
                                let state = Arc::clone(&state);
                                tokio::spawn(async move {
                                    run_rpc(
                                        stream_id,
                                        header,
                                        body,
                                        state,
                                        out_tx,
                                        cmd_rx,
                                    ).await;
                                });
                            }
                            Frame::Cancel { stream_id } => {
                                if let Some(s) = live.get(&stream_id) {
                                    let _ = s.cmd_tx.try_send(StreamCmd::Cancel);
                                }
                            }
                            Frame::Ping { stream_id } => {
                                let _ = out_tx
                                    .send(Outbound::Frame(Frame::Pong { stream_id }))
                                    .await;
                            }
                            Frame::Pong { .. } => {}
                            Frame::Data { stream_id, .. } | Frame::Trailer { stream_id, .. } => {
                                let _ = out_tx.send(Outbound::Frame(Frame::Trailer {
                                    stream_id,
                                    status: Code::InvalidArgument as u32,
                                    message: "client must not send DATA/TRAILER".into(),
                                })).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        log::warn!("websocket recv error: {err}");
                        break;
                    }
                }
            }
        }

        // Drop finished streams whose cmd channel is closed (task ended).
        live.retain(|_, s| !s.cmd_tx.is_closed());
    }

    // Dropping `live` closes cmd channels → tasks see cancel/disconnect.
    drop(live);
    let _ = sink.send(Message::Close(None)).await;
}

async fn run_rpc(
    stream_id: u32,
    header: RequestHeader,
    body: Vec<u8>,
    state: Arc<WsServerState>,
    out_tx: mpsc::Sender<Outbound>,
    mut cmd_rx: mpsc::Receiver<StreamCmd>,
) {
    let result = match header.opcode() {
        Opcode::Subscribe | Opcode::SubscribeWithPolicy => {
            run_subscribe(stream_id, &state, header, &out_tx, &mut cmd_rx).await
        }
        Opcode::Publish => run_publish(&state, header, body).await,
        Opcode::Call => run_call(stream_id, &state, header, body, &out_tx).await,
        Opcode::SendGoal => {
            run_send_goal(stream_id, &state, header, body, &out_tx, &mut cmd_rx).await
        }
    };
    let status = match result {
        Ok(()) => RpcStatus::ok(),
        Err(status) => status,
    };
    let _ = out_tx
        .send(Outbound::Frame(Frame::Trailer {
            stream_id,
            status: status.code_u32(),
            message: status.message().to_string(),
        }))
        .await;
}

async fn run_subscribe(
    stream_id: u32,
    state: &WsServerState,
    header: RequestHeader,
    out_tx: &mpsc::Sender<Outbound>,
    cmd_rx: &mut mpsc::Receiver<StreamCmd>,
) -> Result<(), RpcStatus> {
    let (topic, depth, policy) = match header {
        RequestHeader::Subscribe { topic, qos_depth } => (
            topic,
            qos_depth,
            crate::SubscriptionOverflowPolicy::DropOldest,
        ),
        RequestHeader::SubscribeWithPolicy {
            topic,
            qos_depth,
            overflow,
        } => (topic, qos_depth, overflow),
        _ => return Err(RpcStatus::internal("subscribe header mismatch")),
    };
    let rx = state
        .message
        .open_subscribe_with_policy(topic, depth, policy)?;
    loop {
        let forward = async {
            if !rx.queue.wait_ready().await? {
                return Ok(false);
            }
            // Queue a readiness token, not a frozen payload. The socket writer
            // takes the newest eligible message only when it can actually send.
            let (sent, done) = tokio::sync::oneshot::channel();
            out_tx
                .send(Outbound::Subscription {
                    stream_id,
                    queue: Arc::clone(&rx.queue),
                    sent,
                })
                .await
                .map_err(|_| RpcStatus::internal("subscription connection closed"))?;
            done.await
                .map_err(|_| RpcStatus::internal("subscription writer closed"))?;
            Ok::<_, RpcStatus>(true)
        };
        tokio::select! {
            biased;
            _ = cmd_rx.recv() => break,
            result = forward => if !result? { break; },
        }
    }
    Ok(())
}

async fn run_publish(
    state: &WsServerState,
    header: RequestHeader,
    body: Vec<u8>,
) -> Result<(), RpcStatus> {
    let RequestHeader::Publish { topic } = header else {
        return Err(RpcStatus::internal("publish header mismatch"));
    };
    state.message.publish_message(topic, body).await
}

async fn run_call(
    stream_id: u32,
    state: &WsServerState,
    header: RequestHeader,
    body: Vec<u8>,
    out_tx: &mpsc::Sender<Outbound>,
) -> Result<(), RpcStatus> {
    let RequestHeader::Call {
        service_name,
        timeout_ms,
        request_id,
    } = header
    else {
        return Err(RpcStatus::internal("call header mismatch"));
    };
    let response = state
        .service
        .call_service(service_name, body, request_id, timeout_ms)
        .await?;
    out_tx
        .send(Outbound::Frame(Frame::Data {
            stream_id,
            payload: response,
        }))
        .await
        .map_err(|_| RpcStatus::internal("send call response failed"))?;
    Ok(())
}

async fn run_send_goal(
    stream_id: u32,
    state: &WsServerState,
    header: RequestHeader,
    body: Vec<u8>,
    out_tx: &mpsc::Sender<Outbound>,
    cmd_rx: &mut mpsc::Receiver<StreamCmd>,
) -> Result<(), RpcStatus> {
    let RequestHeader::SendGoal {
        action_name,
        goal_id,
        timeout_ms,
    } = header
    else {
        return Err(RpcStatus::internal("send_goal header mismatch"));
    };
    let mut session: SendGoalSession = state.action.open_send_goal(GoalSpec {
        action_name,
        goal: body,
        goal_id,
        timeout_ms,
    })?;
    loop {
        tokio::select! {
            biased;
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(StreamCmd::Cancel) => {
                        let _ = session.cancel.try_send(Vec::new());
                    }
                    None => {
                        drop(session);
                        return Ok(());
                    }
                }
            }
            item = session.events.recv() => {
                match item {
                    Some(Ok(ev)) => {
                        let is_result = ev.kind == ACTION_KIND_RESULT;
                        let bytes = encode_action_data(stream_id, ev.kind, &ev.body);
                        if out_tx.send(Outbound::Bytes(bytes)).await.is_err() {
                            drop(session);
                            return Ok(());
                        }
                        if is_result {
                            return Ok(());
                        }
                    }
                    Some(Err(status)) => return Err(status),
                    None => break,
                }
            }
        }
    }
    Ok(())
}

type WsSink = futures_util::stream::SplitSink<WebSocket, Message>;

async fn send_frame(sink: &mut WsSink, frame: &Frame) -> Result<(), ()> {
    let bytes = encode_frame(frame).map_err(|_| ())?;
    sink.send(Message::Binary(bytes.into()))
        .await
        .map_err(|_| ())
}
