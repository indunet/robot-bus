//! WebSocket-mode runtime for [`super::Node`]: multiplexed `/ws` client.
//!
//! One WebSocket connection per node carries all subscribe / publish / service /
//! action RPCs (V2 framing with `stream_id`).

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;

use crate::action_bus::{ActionKind, ActionMessage};
use crate::errors::{BusError, Result};
use crate::runtime::callback_group::{CallbackGroup, SubscriptionCallback};
use crate::runtime::executor::ShutdownHandle;
use crate::runtime::node::RawActionFeedbackCallback;
use crate::runtime::registrations::MessageCallback;
use crate::runtime::session::{
    SESSION_BACKOFF_INITIAL, SESSION_BACKOFF_MAX, SESSION_WS_PING_INTERVAL,
    SESSION_WS_PING_MISS_LIMIT, SessionHandle,
};
use crate::runtime::timers::{
    SubscriptionHandle, Timer, TimerCallback, TimerHandle, effective_poll_timeout_ms, tick_timers,
};
use crate::runtime::topic_callbacks::for_each_matching_callback;
use crate::ws_gateway::pb::{
    ActionKind as PbActionKind, GoalCommand, PublishResponse, ServiceCallRequest,
    ServiceCallResponse, SubscribeRequest, TopicMessage,
};
use crate::ws_gateway::rpc_status::Code;
use crate::ws_gateway::ws_frame::{
    Frame, METHOD_CALL, METHOD_PUBLISH, METHOD_SEND_GOAL, METHOD_SUBSCRIBE, decode_frame,
    encode_frame,
};

const DEFAULT_WS_URL: &str = "http://127.0.0.1:15570";
const DEFAULT_SPIN_TIMEOUT_MS: i64 = 250;

#[derive(Debug)]
struct TopicEvent {
    topic: String,
    payload: Vec<u8>,
}

enum ConnCmd {
    Start {
        stream_id: u32,
        method: String,
        payload: Vec<u8>,
        kind: StreamKind,
        reply: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Cancel {
        stream_id: u32,
    },
    Shutdown,
}

enum StreamKind {
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
        event_tx: Sender<Result<ActionMessage>>,
        feedback_callback: Option<RawActionFeedbackCallback>,
    },
}

struct StreamState {
    kind: StreamKind,
}

struct WsConnection {
    cmd_tx: tokio::sync::mpsc::UnboundedSender<ConnCmd>,
    next_stream_id: AtomicU32,
    runtime: Arc<tokio::runtime::Runtime>,
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
    pub(crate) events: Receiver<Result<ActionMessage>>,
    pub(crate) abort: WsCancelHandle,
}

impl WsConnection {
    fn alloc_stream_id(&self) -> u32 {
        self.next_stream_id.fetch_add(2, Ordering::Relaxed)
    }
}

impl WsClientContext {
    pub(crate) fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let payload = TopicMessage {
            topic: topic.to_string(),
            payload: payload.to_vec(),
        }
        .encode_to_vec();
        let data = self.unary(METHOD_PUBLISH, payload)?;
        let _ = PublishResponse::decode(data.as_slice())
            .map_err(|err| BusError::Protocol(format!("decode PublishResponse: {err}")))?;
        Ok(())
    }

    pub(crate) fn call_service(
        &self,
        service_name: &str,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        let payload = ServiceCallRequest {
            service_name: service_name.to_string(),
            request: body.to_vec(),
            request_id: request_id.unwrap_or("").to_string(),
            timeout_ms: timeout_ms_u32(timeout),
        }
        .encode_to_vec();
        let data = self.unary(METHOD_CALL, payload)?;
        let resp = ServiceCallResponse::decode(data.as_slice())
            .map_err(|err| BusError::Protocol(format!("decode ServiceCallResponse: {err}")))?;
        Ok(resp.response)
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
        let payload = GoalCommand {
            action_name: action_name.to_string(),
            goal: body.to_vec(),
            goal_id: goal_id.clone(),
            timeout_ms: timeout_ms_u32(timeout),
        }
        .encode_to_vec();

        let (event_tx, event_rx) = mpsc::channel();
        let stream_id = self.conn.alloc_stream_id();
        self.conn.runtime.block_on(self.start_stream(
            stream_id,
            METHOD_SEND_GOAL,
            payload,
            StreamKind::Action {
                action_name: action_name.to_string(),
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

    fn unary(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        self.conn.runtime.block_on(async {
            let stream_id = self.conn.alloc_stream_id();
            let (data_tx, data_rx) = tokio::sync::oneshot::channel();
            self.start_stream(
                stream_id,
                method,
                payload,
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
        method: &str,
        payload: Vec<u8>,
        kind: StreamKind,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.conn
            .cmd_tx
            .send(ConnCmd::Start {
                stream_id,
                method: method.to_string(),
                payload,
                kind,
                reply: reply_tx,
            })
            .map_err(|_| BusError::Protocol("websocket connection closed".into()))?;
        reply_rx
            .await
            .map_err(|_| BusError::Protocol("websocket request dropped".into()))?
    }
}

struct WsState {
    topic_callbacks: HashMap<String, Vec<SubscriptionCallback>>,
    active_topics: HashSet<String>,
    /// KeepLast depth sent on SubscribeRequest (`0` = gateway default).
    topic_qos: HashMap<String, i32>,
    /// Latest WS stream id for each active topic (for Cancel on destroy).
    topic_stream_ids: HashMap<String, u32>,
    timers: Vec<Timer>,
    next_timer_id: u64,
    next_subscription_id: u64,
}

/// Owns a tokio runtime and dispatches WS subscription / timer callbacks.
pub struct WsRuntime {
    conn: Arc<WsConnection>,
    running: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,
    inbound_tx: Sender<TopicEvent>,
    inbound_rx: Mutex<Receiver<TopicEvent>>,
    state: Arc<Mutex<WsState>>,
}

impl WsRuntime {
    pub fn new(url: impl Into<String>, transport: Option<SessionHandle>) -> Result<Self> {
        let http_url = url.into();
        let ws_url = http_url_to_ws_rpc(&http_url);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("robot-bus-ws")
            .build()
            .map_err(|err| BusError::Protocol(format!("tokio runtime: {err}")))?;
        let runtime = Arc::new(runtime);

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let conn = Arc::new(WsConnection {
            cmd_tx,
            next_stream_id: AtomicU32::new(1),
            runtime: Arc::clone(&runtime),
        });

        runtime.spawn(connection_loop(ws_url, cmd_rx, transport));

        let (inbound_tx, inbound_rx) = mpsc::channel();
        Ok(Self {
            conn,
            running: Arc::new(AtomicBool::new(false)),
            alive: Arc::new(AtomicBool::new(true)),
            inbound_tx,
            inbound_rx: Mutex::new(inbound_rx),
            state: Arc::new(Mutex::new(WsState {
                topic_callbacks: HashMap::new(),
                active_topics: HashSet::new(),
                topic_qos: HashMap::new(),
                topic_stream_ids: HashMap::new(),
                timers: Vec::new(),
                next_timer_id: 1,
                next_subscription_id: 1,
            })),
        })
    }

    pub fn default_url() -> &'static str {
        DEFAULT_WS_URL
    }

    pub fn client_context(&self) -> WsClientContext {
        WsClientContext {
            conn: Arc::clone(&self.conn),
        }
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle::from_flag(Arc::clone(&self.running))
    }

    pub fn shutdown(&self) {
        self.alive.store(false, Ordering::Release);
        self.running.store(false, Ordering::Release);
        let _ = self.conn.cmd_tx.send(ConnCmd::Shutdown);
    }

    pub fn subscribe(
        &self,
        topic: &str,
        callback: MessageCallback,
        group: CallbackGroup,
        qos: Option<crate::QosProfile>,
    ) -> Result<SubscriptionHandle> {
        let mut state = self.lock_state()?;
        let id = state.next_subscription_id;
        state.next_subscription_id += 1;
        state
            .topic_callbacks
            .entry(topic.to_string())
            .or_default()
            .push(SubscriptionCallback {
                id,
                callback,
                group,
            });

        if state.active_topics.insert(topic.to_string()) {
            let depth = qos.map(|q| q.depth()).filter(|d| *d > 0).unwrap_or(0);
            state.topic_qos.insert(topic.to_string(), depth);
            self.spawn_subscription(topic.to_string());
        }
        Ok(SubscriptionHandle { id })
    }

    pub fn destroy_subscription(&self, handle: SubscriptionHandle) -> Result<()> {
        let mut state = self.lock_state()?;
        let mut found_topic: Option<String> = None;
        for (topic, callbacks) in state.topic_callbacks.iter_mut() {
            if let Some(pos) = callbacks.iter().position(|c| c.id == handle.id) {
                callbacks.remove(pos);
                found_topic = Some(topic.clone());
                break;
            }
        }
        let Some(topic) = found_topic else {
            return Err(BusError::Protocol(format!(
                "unknown subscription id {}",
                handle.id
            )));
        };
        let empty = state
            .topic_callbacks
            .get(&topic)
            .map(|c| c.is_empty())
            .unwrap_or(true);
        if empty {
            state.topic_callbacks.remove(&topic);
            state.active_topics.remove(&topic);
            state.topic_qos.remove(&topic);
            if let Some(stream_id) = state.topic_stream_ids.remove(&topic) {
                let _ = self.conn.cmd_tx.send(ConnCmd::Cancel { stream_id });
            }
        }
        Ok(())
    }

    pub fn create_timer(
        &self,
        period: Duration,
        callback: TimerCallback,
        group: CallbackGroup,
    ) -> Result<TimerHandle> {
        let mut state = self.lock_state()?;
        let id = state.next_timer_id;
        state.next_timer_id += 1;
        state.timers.push(Timer::new(id, period, callback, group));
        Ok(TimerHandle { id })
    }

    pub fn cancel_timer(&self, handle: TimerHandle) -> Result<()> {
        let mut state = self.lock_state()?;
        if let Some(timer) = state.timers.iter_mut().find(|t| t.id == handle.id) {
            timer.cancelled = true;
            Ok(())
        } else {
            Err(BusError::Protocol(format!(
                "unknown timer id {}",
                handle.id
            )))
        }
    }

    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.running.store(true, Ordering::Release);
        let timeout_ms = timeout
            .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(DEFAULT_SPIN_TIMEOUT_MS);
        self.poll_once(timeout_ms)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        let _ = self.spin_once(timeout)?;
        Ok(())
    }

    pub fn spin(&self) -> Result<()> {
        self.running.store(true, Ordering::Release);
        while self.running.load(Ordering::Acquire) {
            let timeout_ms = {
                let state = self.lock_state()?;
                effective_poll_timeout_ms(&state.timers, DEFAULT_SPIN_TIMEOUT_MS, Instant::now())
            };
            let _ = self.poll_once(timeout_ms)?;
        }
        Ok(())
    }

    fn poll_once(&self, timeout_ms: i64) -> Result<bool> {
        let mut worked = false;
        {
            let mut state = self.lock_state()?;
            if tick_timers(&mut state.timers, Instant::now(), None) {
                worked = true;
            }
        }

        let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(0) as u64);
        loop {
            let event = {
                let rx = self
                    .inbound_rx
                    .lock()
                    .map_err(|_| BusError::Protocol("ws inbound mutex poisoned".into()))?;
                match rx.try_recv() {
                    Ok(ev) => Some(ev),
                    Err(TryRecvError::Empty) => None,
                    Err(TryRecvError::Disconnected) => {
                        return Err(BusError::Protocol(
                            "ws subscription channel disconnected".into(),
                        ));
                    }
                }
            };

            if let Some(event) = event {
                self.dispatch_topic(&event.topic, &event.payload)?;
                worked = true;
                continue;
            }

            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(worked)
    }

    fn dispatch_topic(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let topic: Arc<str> = topic.into();
        let payload: Arc<[u8]> = payload.to_vec().into();
        let state = self.lock_state()?;
        for_each_matching_callback(&topic, &state.topic_callbacks, |entry| {
            let callback = Arc::clone(&entry.callback);
            let topic = Arc::clone(&topic);
            let payload = Arc::clone(&payload);
            entry.group.run(None, move || callback(&topic, &payload));
        });
        Ok(())
    }

    fn spawn_subscription(&self, topic: String) {
        let tx = self.inbound_tx.clone();
        let alive = Arc::clone(&self.alive);
        let conn = Arc::clone(&self.conn);
        let state = Arc::clone(&self.state);
        self.conn.runtime.spawn(async move {
            let mut backoff = SESSION_BACKOFF_INITIAL;
            loop {
                {
                    let Ok(guard) = state.lock() else {
                        break;
                    };
                    if !guard.active_topics.contains(&topic) {
                        break;
                    }
                }
                if !alive.load(Ordering::Acquire) {
                    break;
                }
                let stream_id = conn.alloc_stream_id();
                if let Ok(mut guard) = state.lock() {
                    guard.topic_stream_ids.insert(topic.clone(), stream_id);
                }
                let qos_depth = {
                    let Ok(guard) = state.lock() else {
                        break;
                    };
                    guard.topic_qos.get(&topic).copied().unwrap_or(0)
                };
                let payload = SubscribeRequest {
                    topic: topic.clone(),
                    qos_depth,
                }
                .encode_to_vec();
                let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                let (done_tx, done_rx) = tokio::sync::oneshot::channel();
                if conn
                    .cmd_tx
                    .send(ConnCmd::Start {
                        stream_id,
                        method: METHOD_SUBSCRIBE.to_string(),
                        payload,
                        kind: StreamKind::Subscribe {
                            topic: topic.clone(),
                            tx: tx.clone(),
                            done: Some(done_tx),
                        },
                        reply: reply_tx,
                    })
                    .is_err()
                {
                    break;
                }
                match reply_rx.await {
                    Ok(Ok(())) => {
                        backoff = SESSION_BACKOFF_INITIAL;
                        let _ = done_rx.await;
                    }
                    Ok(Err(err)) => log::warn!("ws subscribe '{topic}' start failed: {err}"),
                    Err(_) => break,
                }
                if !alive.load(Ordering::Acquire) {
                    break;
                }
                {
                    let Ok(guard) = state.lock() else {
                        break;
                    };
                    if !guard.active_topics.contains(&topic) {
                        break;
                    }
                }
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff.saturating_mul(2), SESSION_BACKOFF_MAX);
            }
        });
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, WsState>> {
        self.state
            .lock()
            .map_err(|_| BusError::Protocol("ws state mutex poisoned".into()))
    }
}

impl Drop for WsRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

async fn connection_loop(
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
                    Some(ConnCmd::Start { stream_id, method, payload, kind, reply }) => {
                        let frame = Frame::Request { stream_id, method, payload };
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
                    if let Ok(msg) = TopicMessage::decode(payload.as_slice()) {
                        let _ = tx.send(TopicEvent {
                            topic: msg.topic,
                            payload: msg.payload,
                        });
                    }
                }
                StreamKind::Unary { data, .. } => {
                    *data = Some(payload);
                }
                StreamKind::Action {
                    action_name,
                    event_tx,
                    feedback_callback,
                } => {
                    if let Ok(ev) = crate::ws_gateway::pb::ActionEvent::decode(payload.as_slice()) {
                        match pb_action_kind(ev.kind) {
                            Ok(kind) => {
                                let done = kind == ActionKind::Result;
                                let message = ActionMessage {
                                    action_name: ev.action_name,
                                    goal_id: ev.goal_id,
                                    kind,
                                    body: ev.body,
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
                        let _ = action_name;
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
                    let _ = reply.send(match data {
                        Some(d) => Ok(d),
                        None => Err(BusError::Protocol("unary trailer without DATA".into())),
                    });
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

pub fn http_url_to_ws_rpc(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        return if trimmed.ends_with("/ws") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/ws")
        };
    }
    if let Some(rest) = trimmed.strip_prefix("https://") {
        return format!("wss://{rest}/ws");
    }
    if let Some(rest) = trimmed.strip_prefix("http://") {
        return format!("ws://{rest}/ws");
    }
    format!("ws://{trimmed}/ws")
}

fn timeout_ms_u32(timeout: Option<Duration>) -> u32 {
    match timeout {
        None => 0,
        Some(d) => d.as_millis().min(u32::MAX as u128) as u32,
    }
}

fn pb_action_kind(kind: i32) -> Result<ActionKind> {
    match PbActionKind::try_from(kind) {
        Ok(PbActionKind::Goal) => Ok(ActionKind::Goal),
        Ok(PbActionKind::Feedback) => Ok(ActionKind::Feedback),
        Ok(PbActionKind::Result) => Ok(ActionKind::Result),
        Ok(PbActionKind::Cancel) => Ok(ActionKind::Cancel),
        Ok(PbActionKind::Unspecified) | Err(_) => Err(BusError::Protocol(format!(
            "unknown ws action kind: {kind}"
        ))),
    }
}

fn map_rpc_status(status: u32, message: &str) -> BusError {
    match Code::from_u32(status) {
        Code::DeadlineExceeded => BusError::Timeout(message.to_string()),
        Code::NotFound => {
            if let Some(rest) = message.strip_prefix("no goal ") {
                BusError::NoGoal {
                    goal_id: rest.trim_matches('\'').to_string(),
                }
            } else {
                BusError::Protocol(format!("rpc {status}: {message}"))
            }
        }
        Code::Unavailable => {
            if let Some(rest) = message.strip_prefix("no worker for ") {
                BusError::NoWorker {
                    name: rest.trim_matches('\'').to_string(),
                }
            } else if let Some(rest) = message.strip_prefix("worker died for ") {
                BusError::WorkerDied {
                    name: rest.trim_matches('\'').to_string(),
                }
            } else {
                BusError::Protocol(format!("rpc {status}: {message}"))
            }
        }
        _ => BusError::Protocol(format!("rpc {status}: {message}")),
    }
}
