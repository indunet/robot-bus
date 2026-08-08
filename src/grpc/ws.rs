//! Browser gRPC-over-WebSocket: one WebSocket connection = one RPC (V1).

use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use tonic::Status;

use super::action::ActionGatewayService;
use super::message::MessageGatewayService;
use super::pb::{
    ActionKind, GoalCommand, PublishResponse, ServiceCallRequest, ServiceCallResponse,
    SubscribeRequest, TopicMessage,
};
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

async fn handle_socket(socket: WebSocket, state: Arc<WsGatewayState>) {
    let (mut sink, mut stream) = socket.split();

    let first = match stream.next().await {
        Some(Ok(Message::Binary(bin))) => bin,
        Some(Ok(Message::Close(_))) | None => return,
        Some(Ok(_)) => {
            let _ = send_trailer(&mut sink, Status::invalid_argument("expected binary REQUEST")).await;
            return;
        }
        Some(Err(err)) => {
            log::warn!("websocket recv error: {err}");
            return;
        }
    };

    let frame = match decode_frame(&first) {
        Ok(frame) => frame,
        Err(err) => {
            let _ = send_trailer(&mut sink, Status::invalid_argument(err.to_string())).await;
            return;
        }
    };

    let Frame::Request { method, payload } = frame else {
        let _ = send_trailer(&mut sink, Status::invalid_argument("first frame must be REQUEST")).await;
        return;
    };

    match method.as_str() {
        METHOD_SUBSCRIBE => {
            if let Err(status) = run_subscribe(&mut sink, &mut stream, &state, payload).await {
                let _ = send_trailer(&mut sink, status).await;
            }
        }
        METHOD_PUBLISH => {
            let status = run_publish(&mut sink, &state, payload).await;
            let _ = send_trailer(&mut sink, status).await;
        }
        METHOD_CALL => {
            let status = run_call(&mut sink, &state, payload).await;
            let _ = send_trailer(&mut sink, status).await;
        }
        METHOD_SEND_GOAL => {
            if let Err(status) = run_send_goal(&mut sink, &mut stream, &state, payload).await {
                let _ = send_trailer(&mut sink, status).await;
            }
        }
        other => {
            let _ = send_trailer(
                &mut sink,
                Status::unimplemented(format!("unknown method '{other}'")),
            )
            .await;
        }
    }

    let _ = sink.send(Message::Close(None)).await;
}

type WsSink = futures_util::stream::SplitSink<WebSocket, Message>;
type WsStream = futures_util::stream::SplitStream<WebSocket>;

async fn run_subscribe(
    sink: &mut WsSink,
    inbound: &mut WsStream,
    state: &WsGatewayState,
    payload: Vec<u8>,
) -> Result<(), Status> {
    let req = SubscribeRequest::decode(payload.as_slice())
        .map_err(|err| Status::invalid_argument(format!("decode SubscribeRequest: {err}")))?;
    let mut rx = state.message.open_subscribe(req.topic)?;

    loop {
        tokio::select! {
            biased;
            client_msg = inbound.next() => {
                match client_msg {
                    Some(Ok(Message::Binary(bin))) => {
                        if matches!(decode_frame(&bin), Ok(Frame::Cancel)) {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            item = rx.recv() => {
                match item {
                    Some(Ok(msg)) => {
                        let bytes = msg.encode_to_vec();
                        send_frame(sink, &Frame::Data { payload: bytes }).await?;
                    }
                    Some(Err(status)) => return Err(status),
                    None => break,
                }
            }
        }
    }
    send_trailer(sink, Status::new(tonic::Code::Ok, "")).await
}

async fn run_publish(
    sink: &mut WsSink,
    state: &WsGatewayState,
    payload: Vec<u8>,
) -> Status {
    let msg = match TopicMessage::decode(payload.as_slice()) {
        Ok(msg) => msg,
        Err(err) => return Status::invalid_argument(format!("decode TopicMessage: {err}")),
    };
    match state.message.publish_message(msg).await {
        Ok(()) => {
            let bytes = PublishResponse {}.encode_to_vec();
            if send_frame(sink, &Frame::Data { payload: bytes })
                .await
                .is_err()
            {
                return Status::internal("send publish ack failed");
            }
            Status::new(tonic::Code::Ok, "")
        }
        Err(status) => status,
    }
}

async fn run_call(sink: &mut WsSink, state: &WsGatewayState, payload: Vec<u8>) -> Status {
    let req = match ServiceCallRequest::decode(payload.as_slice()) {
        Ok(req) => req,
        Err(err) => {
            return Status::invalid_argument(format!("decode ServiceCallRequest: {err}"));
        }
    };
    match state.service.call_service(req).await {
        Ok(response) => {
            let bytes = ServiceCallResponse { response }.encode_to_vec();
            if send_frame(sink, &Frame::Data { payload: bytes })
                .await
                .is_err()
            {
                return Status::internal("send call response failed");
            }
            Status::new(tonic::Code::Ok, "")
        }
        Err(status) => status,
    }
}

async fn run_send_goal(
    sink: &mut WsSink,
    inbound: &mut WsStream,
    state: &WsGatewayState,
    payload: Vec<u8>,
) -> Result<(), Status> {
    let goal = GoalCommand::decode(payload.as_slice())
        .map_err(|err| Status::invalid_argument(format!("decode GoalCommand: {err}")))?;
    let mut session = state.action.open_send_goal(goal)?;

    loop {
        tokio::select! {
            biased;
            client_msg = inbound.next() => {
                match client_msg {
                    Some(Ok(Message::Binary(bin))) => {
                        // Soft cancel: ask the worker to stop, keep the socket open
                        // for FEEDBACK/RESULT (normal bidi path).
                        if matches!(decode_frame(&bin), Ok(Frame::Cancel)) {
                            let _ = session.cancel.try_send(Vec::new());
                        }
                    }
                    // Hard disconnect: drop the session → gateway submits cancel and abandons.
                    Some(Ok(Message::Close(_))) | None => {
                        drop(session);
                        return send_trailer(sink, Status::new(tonic::Code::Ok, "")).await;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => {
                        drop(session);
                        return Ok(());
                    }
                }
            }
            item = session.events.recv() => {
                match item {
                    Some(Ok(ev)) => {
                        let bytes = ev.encode_to_vec();
                        send_frame(sink, &Frame::Data { payload: bytes }).await?;
                        if ev.kind == i32::from(ActionKind::Result) {
                            return send_trailer(sink, Status::new(tonic::Code::Ok, "")).await;
                        }
                    }
                    Some(Err(status)) => return Err(status),
                    None => break,
                }
            }
        }
    }
    send_trailer(sink, Status::new(tonic::Code::Ok, "")).await
}

async fn send_frame(sink: &mut WsSink, frame: &Frame) -> Result<(), Status> {
    let bytes = encode_frame(frame).map_err(|err| Status::internal(err.to_string()))?;
    sink.send(Message::Binary(bytes.into()))
        .await
        .map_err(|err| Status::internal(format!("websocket send: {err}")))
}

async fn send_trailer(sink: &mut WsSink, status: Status) -> Result<(), Status> {
    let frame = Frame::Trailer {
        status: status.code() as u32,
        message: status.message().to_string(),
    };
    send_frame(sink, &frame).await
}
