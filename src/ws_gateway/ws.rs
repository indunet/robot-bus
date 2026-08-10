//! Multiplexed WebSocket RPC gateway (V2: one connection, many streams).

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use tokio::sync::mpsc;

use super::action::{ActionGatewayService, SendGoalSession};
use super::message::MessageGatewayService;
use super::pb::ActionKind;
use super::rpc_status::{Code, RpcStatus};
use super::service::ServiceGatewayService;
use super::ws_frame::{
    Frame, METHOD_CALL, METHOD_PUBLISH, METHOD_SEND_GOAL, METHOD_SUBSCRIBE, decode_frame,
    encode_frame,
};

#[derive(Clone)]
pub struct WsGatewayState {
    pub message: MessageGatewayService,
    pub service: ServiceGatewayService,
    pub action: ActionGatewayService,
}

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsGatewayState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

enum Outbound {
    Frame(Frame),
}

enum StreamCmd {
    Cancel,
}

struct LiveStream {
    cmd_tx: mpsc::Sender<StreamCmd>,
}

async fn handle_socket(socket: WebSocket, state: Arc<WsGatewayState>) {
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
                            Frame::Request { stream_id, method, payload } => {
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
                                        method,
                                        payload,
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
    method: String,
    payload: Vec<u8>,
    state: Arc<WsGatewayState>,
    out_tx: mpsc::Sender<Outbound>,
    mut cmd_rx: mpsc::Receiver<StreamCmd>,
) {
    let result = match method.as_str() {
        METHOD_SUBSCRIBE => run_subscribe(stream_id, &state, payload, &out_tx, &mut cmd_rx).await,
        METHOD_PUBLISH => run_publish(stream_id, &state, payload, &out_tx).await,
        METHOD_CALL => run_call(stream_id, &state, payload, &out_tx).await,
        METHOD_SEND_GOAL => run_send_goal(stream_id, &state, payload, &out_tx, &mut cmd_rx).await,
        other => Err(RpcStatus::unimplemented(format!("unknown method '{other}'"))),
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
    state: &WsGatewayState,
    payload: Vec<u8>,
    out_tx: &mpsc::Sender<Outbound>,
    cmd_rx: &mut mpsc::Receiver<StreamCmd>,
) -> Result<(), RpcStatus> {
    let (_topic, mut rx) = state.message.handle_subscribe_request(&payload)?;
    loop {
        tokio::select! {
            biased;
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(StreamCmd::Cancel) | None => break,
                }
            }
            item = rx.recv() => {
                match item {
                    Some(Ok(msg)) => {
                        let bytes = msg.encode_to_vec();
                        if out_tx
                            .send(Outbound::Frame(Frame::Data {
                                stream_id,
                                payload: bytes,
                            }))
                            .await
                            .is_err()
                        {
                            break;
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

async fn run_publish(
    stream_id: u32,
    state: &WsGatewayState,
    payload: Vec<u8>,
    out_tx: &mpsc::Sender<Outbound>,
) -> Result<(), RpcStatus> {
    let resp = state.message.handle_publish(&payload).await?;
    let bytes = resp.encode_to_vec();
    out_tx
        .send(Outbound::Frame(Frame::Data {
            stream_id,
            payload: bytes,
        }))
        .await
        .map_err(|_| RpcStatus::internal("send publish ack failed"))?;
    Ok(())
}

async fn run_call(
    stream_id: u32,
    state: &WsGatewayState,
    payload: Vec<u8>,
    out_tx: &mpsc::Sender<Outbound>,
) -> Result<(), RpcStatus> {
    let resp = state.service.handle_call(&payload).await?;
    let bytes = resp.encode_to_vec();
    out_tx
        .send(Outbound::Frame(Frame::Data {
            stream_id,
            payload: bytes,
        }))
        .await
        .map_err(|_| RpcStatus::internal("send call response failed"))?;
    Ok(())
}

async fn run_send_goal(
    stream_id: u32,
    state: &WsGatewayState,
    payload: Vec<u8>,
    out_tx: &mpsc::Sender<Outbound>,
    cmd_rx: &mut mpsc::Receiver<StreamCmd>,
) -> Result<(), RpcStatus> {
    let mut session: SendGoalSession = state.action.handle_send_goal(&payload)?;
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
                        let is_result = ev.kind == i32::from(ActionKind::Result);
                        let bytes = ev.encode_to_vec();
                        if out_tx
                            .send(Outbound::Frame(Frame::Data {
                                stream_id,
                                payload: bytes,
                            }))
                            .await
                            .is_err()
                        {
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
    sink.send(Message::Binary(bytes.into())).await.map_err(|_| ())
}
