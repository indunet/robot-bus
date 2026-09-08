//! Multiplexed WebSocket connection: stream start/cancel and inbound demux.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;

use crate::action_bus::{ActionKind, ActionMessage};
use crate::errors::{BusError, Result};
use crate::runtime::node::RawActionFeedbackCallback;
use crate::runtime::session::{
    SESSION_BACKOFF_INITIAL, SESSION_BACKOFF_MAX, SESSION_WS_PING_INTERVAL,
    SESSION_WS_PING_MISS_LIMIT, SessionHandle,
};
use crate::ws::ws_frame::{
    Frame, RequestHeader, decode_action_data, decode_frame, decode_subscribe_data, encode_frame,
};

use super::status::{map_rpc_status, ws_action_kind};

#[derive(Debug)]
pub(super) struct TopicEvent {
    pub(super) topic: String,
    pub(super) payload: Vec<u8>,
}

pub(super) enum ConnCmd {
    Start {
        stream_id: u32,
        header: RequestHeader,
        body: Vec<u8>,
        kind: StreamKind,
        reply: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Cancel {
        stream_id: u32,
    },
    Shutdown,
}

pub(super) enum StreamKind {
    Subscribe {
        #[allow(dead_code)]
        topic: String,
        tx: Sender<TopicEvent>,
        done: Option<tokio::sync::oneshot::Sender<()>>,
    },
    Unary {
        reply: tokio::sync::oneshot::Sender<Result<Vec<u8>>>,
        data: Option<Vec<u8>>,
    },
    Action {
        action_name: String,
        goal_id: String,
        event_tx: Sender<Result<ActionMessage>>,
        feedback_callback: Option<RawActionFeedbackCallback>,
    },
}

struct StreamState {
    kind: StreamKind,
}

pub(super) struct WsConnection {
    pub(super) cmd_tx: tokio::sync::mpsc::UnboundedSender<ConnCmd>,
    pub(super) next_stream_id: AtomicU32,
    pub(super) handle: tokio::runtime::Handle,
}

/// Shared handle used by WS service / action clients.
#[derive(Clone)]
pub(crate) struct WsClientContext {
    conn: Arc<WsConnection>,
}

/// Soft-cancel handle for a live SendGoal stream (replaces tonic AbortHandle).
#[derive(Clone)]
pub(crate) struct WsCancelHandle {
    conn: Arc<WsConnection>,
    stream_id: u32,
    cancelled: Arc<AtomicBool>,
}

impl WsCancelHandle {
    pub(crate) fn abort(&self) {
        if self
            .cancelled
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let _ = self.conn.cmd_tx.send(ConnCmd::Cancel {
                stream_id: self.stream_id,
            });
        }
    }
}

pub(crate) struct WsGoalSession {
    pub(crate) goal_id: String,
    pub(crate) events: std::sync::mpsc::Receiver<Result<ActionMessage>>,
    pub(crate) abort: WsCancelHandle,
}

impl WsConnection {
    pub(super) fn alloc_stream_id(&self) -> u32 {
        self.next_stream_id.fetch_add(2, Ordering::Relaxed)
    }
}

impl WsClientContext {
    pub(super) fn new(conn: Arc<WsConnection>) -> Self {
        Self { conn }
    }

    pub(crate) fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let _ = self.unary(
            RequestHeader::Publish {
                topic: topic.to_string(),
            },
            payload.to_vec(),
        )?;
        Ok(())
    }

    pub(crate) fn call_service(
        &self,
        service_name: &str,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        self.unary(
            RequestHeader::Call {
                service_name: service_name.to_string(),
                timeout_ms: timeout_ms_u32(timeout),
                request_id: request_id.unwrap_or("").to_string(),
            },
            body.to_vec(),
        )
    }

    pub(crate) fn send_goal(
        &self,
        action_name: &str,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
        feedback_callback: Option<RawActionFeedbackCallback>,
    ) -> Result<WsGoalSession> {
        let goal_id = goal_id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let stream_id = self.conn.alloc_stream_id();
        self.conn.handle.block_on(self.start_stream(
            stream_id,
            RequestHeader::SendGoal {
                action_name: action_name.to_string(),
                goal_id: goal_id.clone(),
                timeout_ms: timeout_ms_u32(timeout),
            },
            body.to_vec(),
            StreamKind::Action {
                action_name: action_name.to_string(),
                goal_id: goal_id.clone(),
                event_tx,
                feedback_callback,
            },
        ))?;

        Ok(WsGoalSession {
            goal_id,
            events: event_rx,
            abort: WsCancelHandle {
                conn: Arc::clone(&self.conn),
                stream_id,
                cancelled: Arc::new(AtomicBool::new(false)),
            },
        })
    }

    fn unary(&self, header: RequestHeader, body: Vec<u8>) -> Result<Vec<u8>> {
        self.conn.handle.block_on(async {
            let stream_id = self.conn.alloc_stream_id();
            let (data_tx, data_rx) = tokio::sync::oneshot::channel();
            self.start_stream(
                stream_id,
                header,
                body,
                StreamKind::Unary {
                    reply: data_tx,
                    data: None,
                },
            )
            .await?;
            data_rx
                .await
                .map_err(|_| BusError::Protocol("websocket unary cancelled".into()))?
        })
    }

    async fn start_stream(
        &self,
        stream_id: u32,
        header: RequestHeader,
        body: Vec<u8>,
        kind: StreamKind,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.conn
            .cmd_tx
            .send(ConnCmd::Start {
                stream_id,
                header,
                body,
                kind,
                reply: reply_tx,
            })
            .map_err(|_| BusError::Protocol("websocket connection closed".into()))?;
        reply_rx
            .await
            .map_err(|_| BusError::Protocol("websocket request dropped".into()))?
    }
}

fn timeout_ms_u32(timeout: Option<Duration>) -> u32 {
    match timeout {
        None => 0,
        Some(d) => d.as_millis().min(u32::MAX as u128) as u32,
    }
}

pub(super) async fn connection_loop(
    ws_url: String,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<ConnCmd>,
    transport: Option<SessionHandle>,
) {
    let mut backoff = SESSION_BACKOFF_INITIAL;
    loop {
        match connect_async(&ws_url).await {
            Ok((ws, _)) => {
                backoff = SESSION_BACKOFF_INITIAL;
                if let Some(t) = &transport {
                    t.note_transport_up("ws open");
                }
                match run_ws_connection(ws, &mut cmd_rx).await {
                    WsLoopExit::Shutdown => return,
                    WsLoopExit::Disconnected => {
                        log::warn!("ws {ws_url} disconnected; reconnecting");
                        if let Some(t) = &transport {
                            t.note_transport_down("ws closed");
                        }
                    }
                }
            }
            Err(err) => {
                log::debug!("ws connect {ws_url} failed: {err}");
                if let Some(t) = &transport {
                    t.note_transport_down("ws connect failed");
                }
                if fail_pending_starts(
                    &mut cmd_rx,
                    BusError::Protocol(format!("ws connect failed: {err}")),
                )
                .await
                {
                    return;
                }
            }
        }
        match backoff_or_shutdown(&mut cmd_rx, backoff).await {
            WsLoopExit::Shutdown => return,
            WsLoopExit::Disconnected => {
                backoff = std::cmp::min(backoff.saturating_mul(2), SESSION_BACKOFF_MAX);
            }
        }
    }
}

enum WsLoopExit {
    Shutdown,
    Disconnected,
}

async fn fail_pending_starts(
    cmd_rx: &mut tokio::sync::mpsc::UnboundedReceiver<ConnCmd>,
    err: BusError,
) -> bool {
    while let Ok(cmd) = cmd_rx.try_recv() {
        match cmd {
            ConnCmd::Start { reply, .. } => {
                let _ = reply.send(Err(BusError::Protocol(err.to_string())));
            }
            ConnCmd::Shutdown => return true,
            ConnCmd::Cancel { .. } => {}
        }
    }
    false
}

async fn backoff_or_shutdown(
    cmd_rx: &mut tokio::sync::mpsc::UnboundedReceiver<ConnCmd>,
    backoff: Duration,
) -> WsLoopExit {
    let sleep = tokio::time::sleep(backoff);
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => return WsLoopExit::Disconnected,
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(ConnCmd::Shutdown) | None => return WsLoopExit::Shutdown,
                    Some(ConnCmd::Start { reply, .. }) => {
                        let _ = reply.send(Err(BusError::Protocol(
                            "websocket reconnecting".into(),
                        )));
                    }
                    Some(ConnCmd::Cancel { .. }) => {}
                }
            }
        }
    }
}

async fn run_ws_connection(
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    cmd_rx: &mut tokio::sync::mpsc::UnboundedReceiver<ConnCmd>,
) -> WsLoopExit {
    let (mut sink, mut stream) = ws.split();
    let mut streams: HashMap<u32, StreamState> = HashMap::new();
    let mut ping_interval = tokio::time::interval(SESSION_WS_PING_INTERVAL);
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    ping_interval.tick().await;
    let mut awaiting_pong = false;
    let mut ping_misses: u32 = 0;
    let mut heartbeat = true;

    let exit = loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(ConnCmd::Start { stream_id, header, body, kind, reply }) => {
                        let frame = Frame::Request { stream_id, header, body };
                        match encode_frame(&frame) {
                            Ok(bytes) => {
                                if let Err(err) = sink.send(WsMessage::Binary(bytes.into())).await {
                                    let _ = reply.send(Err(BusError::Protocol(format!(
                                        "ws send failed: {err}"
                                    ))));
                                    break WsLoopExit::Disconnected;
                                }
                                streams.insert(stream_id, StreamState { kind });
                                let _ = reply.send(Ok(()));
                            }
                            Err(err) => {
                                let _ = reply.send(Err(BusError::Protocol(err.to_string())));
                            }
                        }
                    }
                    Some(ConnCmd::Cancel { stream_id }) => {
                        let frame = Frame::Cancel { stream_id };
                        if let Ok(bytes) = encode_frame(&frame) {
                            let _ = sink.send(WsMessage::Binary(bytes.into())).await;
                        }
                    }
                    Some(ConnCmd::Shutdown) | None => break WsLoopExit::Shutdown,
                }
            }
            _ = ping_interval.tick(), if heartbeat => {
                if awaiting_pong {
                    ping_misses = ping_misses.saturating_add(1);
                    if ping_misses >= SESSION_WS_PING_MISS_LIMIT {
                        log::warn!("ws ping timeout; reconnecting");
                        break WsLoopExit::Disconnected;
                    }
                }
                awaiting_pong = true;
                match encode_frame(&Frame::Ping { stream_id: 0 }) {
                    Ok(bytes) => {
                        if sink.send(WsMessage::Binary(bytes.into())).await.is_err() {
                            break WsLoopExit::Disconnected;
                        }
                    }
                    Err(_) => break WsLoopExit::Disconnected,
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(WsMessage::Binary(bin))) => {
                        let frame = match decode_frame(&bin) {
                            Ok(f) => f,
                            Err(err) => {
                                log::warn!("ws bad frame: {err}");
                                continue;
                            }
                        };
                        match &frame {
                            Frame::Pong { .. } => {
                                awaiting_pong = false;
                                ping_misses = 0;
                            }
                            Frame::Ping { stream_id } => {
                                if let Ok(bytes) = encode_frame(&Frame::Pong { stream_id: *stream_id }) {
                                    if sink.send(WsMessage::Binary(bytes.into())).await.is_err() {
                                        break WsLoopExit::Disconnected;
                                    }
                                }
                            }
                            Frame::Trailer { stream_id: 0, status, message }
                                if *status != 0 && message == "unknown opcode 5" => {
                                log::warn!("broker does not support subscription overflow policies: {message}");
                                let ids: Vec<_> = streams.keys().copied().collect();
                                for stream_id in ids {
                                    handle_inbound_frame(&mut streams, Frame::Trailer { stream_id, status: *status, message: message.clone() });
                                }
                            }
                            Frame::Trailer { stream_id: 0, .. } => {
                                heartbeat = false;
                                awaiting_pong = false;
                            }
                            _ => handle_inbound_frame(&mut streams, frame),
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) | None => break WsLoopExit::Disconnected,
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        log::warn!("ws recv error: {err}");
                        break WsLoopExit::Disconnected;
                    }
                }
            }
        }
    };

    for (_, st) in streams.drain() {
        fail_stream(st.kind, BusError::Protocol("websocket closed".into()));
    }
    exit
}

fn handle_inbound_frame(streams: &mut HashMap<u32, StreamState>, frame: Frame) {
    let stream_id = frame.stream_id();
    match frame {
        Frame::Data { stream_id, payload } => {
            let Some(st) = streams.get_mut(&stream_id) else {
                return;
            };
            match &mut st.kind {
                StreamKind::Subscribe { tx, .. } => {
                    if let Ok((topic, payload)) = decode_subscribe_data(&payload) {
                        let _ = tx.send(TopicEvent { topic, payload });
                    }
                }
                StreamKind::Unary { data, .. } => {
                    *data = Some(payload);
                }
                StreamKind::Action {
                    action_name,
                    goal_id,
                    event_tx,
                    feedback_callback,
                } => {
                    if let Ok((kind_u8, body)) = decode_action_data(&payload) {
                        match ws_action_kind(kind_u8) {
                            Ok(kind) => {
                                let done = kind == ActionKind::Result;
                                let message = ActionMessage {
                                    action_name: action_name.clone(),
                                    goal_id: goal_id.clone(),
                                    kind,
                                    body,
                                };
                                if kind == ActionKind::Feedback {
                                    if let Some(callback) = feedback_callback {
                                        callback(&message);
                                    }
                                }
                                if done {
                                    if let Some(err) =
                                        crate::errors::parse_error_body(&message.body)
                                    {
                                        let _ = event_tx.send(Err(err));
                                        return;
                                    }
                                }
                                let _ = event_tx.send(Ok(message));
                                if done {
                                    // Keep until TRAILER for cleanup.
                                }
                            }
                            Err(err) => {
                                let _ = event_tx.send(Err(err));
                            }
                        }
                    }
                }
            }
        }
        Frame::Trailer {
            stream_id,
            status,
            message,
        } => {
            let Some(st) = streams.remove(&stream_id) else {
                return;
            };
            if status != 0 {
                fail_stream(st.kind, map_rpc_status(status, &message));
                return;
            }
            match st.kind {
                StreamKind::Unary { reply, data } => {
                    let _ = reply.send(Ok(data.unwrap_or_default()));
                }
                StreamKind::Subscribe { done, .. } => {
                    if let Some(done) = done {
                        let _ = done.send(());
                    }
                }
                StreamKind::Action { event_tx, .. } => {
                    let _ = event_tx;
                }
            }
        }
        Frame::Request { .. } | Frame::Cancel { .. } | Frame::Ping { .. } | Frame::Pong { .. } => {}
    }
    let _ = stream_id;
}

fn fail_stream(kind: StreamKind, err: BusError) {
    match kind {
        StreamKind::Unary { reply, .. } => {
            let _ = reply.send(Err(err));
        }
        StreamKind::Action { event_tx, .. } => {
            let _ = event_tx.send(Err(err));
        }
        StreamKind::Subscribe { done, .. } => {
            if let Some(done) = done {
                let _ = done.send(());
            }
        }
    }
}
