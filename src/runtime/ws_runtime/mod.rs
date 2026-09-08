//! WebSocket-mode runtime for [`super::Node`]: multiplexed `/ws-rpc` client.
//!
//! One WebSocket connection per node carries all subscribe / publish / service /
//! action RPCs (V3 framing with `stream_id`).

mod conn;
mod status;

pub(crate) use conn::{WsCancelHandle, WsClientContext};

use crate::QosProfile;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::errors::{BusError, Result};
use crate::runtime::callback_group::{CallbackGroup, SubscriptionCallback};
use crate::runtime::executor::ShutdownHandle;
use crate::runtime::registrations::MessageCallback;
use crate::runtime::session::{SESSION_BACKOFF_INITIAL, SESSION_BACKOFF_MAX, SessionHandle};
use crate::runtime::timers::{
    SubscriptionHandle, Timer, TimerCallback, TimerHandle, effective_poll_timeout_ms, tick_timers,
};
use crate::runtime::topic_callbacks::for_each_matching_callback;
use crate::ws::ws_frame::RequestHeader;

use conn::{ConnCmd, StreamKind, TopicEvent, WsConnection, connection_loop};

const DEFAULT_WS_URL: &str = "http://127.0.0.1:15560";
const DEFAULT_SPIN_TIMEOUT_MS: i64 = 250;

struct WsState {
    topic_callbacks: HashMap<String, Vec<SubscriptionCallback>>,
    active_topics: HashSet<String>,
    /// KeepLast depth sent on Subscribe REQUEST (`0` = server default).
    topic_qos: HashMap<String, QosProfile>,
    /// Latest WS stream id for each active topic (for Cancel on destroy).
    topic_stream_ids: HashMap<String, u32>,
    timers: Vec<Timer>,
    next_timer_id: u64,
    next_subscription_id: u64,
}

/// Owns a tokio runtime and dispatches WS subscription / timer callbacks.
///
/// `runtime` is declared first so it is dropped **last** (after `conn` and
/// spawned tasks that only hold a [`tokio::runtime::Handle`], not the Runtime).
pub struct WsRuntime {
    /// Owned runtime; tasks only hold a Handle. Declared first so it drops last.
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
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

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let conn = Arc::new(WsConnection {
            cmd_tx,
            next_stream_id: AtomicU32::new(1),
            handle: runtime.handle().clone(),
        });

        runtime.spawn(connection_loop(ws_url, cmd_rx, transport));

        let (inbound_tx, inbound_rx) = mpsc::channel();
        Ok(Self {
            runtime,
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
        WsClientContext::new(Arc::clone(&self.conn))
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
        let requested = qos.unwrap_or(QosProfile::keep_last(0));
        if let Some(existing) = state.topic_qos.get(topic) {
            if crate::runtime::ws_subscribe_queue_capacity(existing.depth())
                != crate::runtime::ws_subscribe_queue_capacity(requested.depth())
            {
                return Err(BusError::Protocol(format!(
                    "conflicting KeepLast depth for {topic}"
                )));
            }
        }
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
            state.topic_qos.insert(topic.to_string(), requested);
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
        let payload: Arc<[u8]> = payload.to_vec().into();
        let state = self.lock_state()?;
        for_each_matching_callback(topic, &state.topic_callbacks, |entry| {
            let callback = Arc::clone(&entry.callback);
            let payload = Arc::clone(&payload);
            entry.group.run(None, move || callback(&payload));
        });
        Ok(())
    }

    fn spawn_subscription(&self, topic: String) {
        let tx = self.inbound_tx.clone();
        let alive = Arc::clone(&self.alive);
        let conn = Arc::clone(&self.conn);
        let state = Arc::clone(&self.state);
        self.conn.handle.spawn(async move {
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
                    guard
                        .topic_qos
                        .get(&topic)
                        .copied()
                        .unwrap_or(QosProfile::keep_last(0))
                };
                // Explicit opcode prevents an older server silently using drop-newest.
                let header = RequestHeader::SubscribeWithPolicy {
                    topic: topic.clone(),
                    qos_depth: qos_depth.depth(),
                    overflow: crate::SubscriptionOverflowPolicy::DropOldest,
                };
                let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                let (done_tx, done_rx) = tokio::sync::oneshot::channel();
                if conn
                    .cmd_tx
                    .send(ConnCmd::Start {
                        stream_id,
                        header,
                        body: Vec::new(),
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

pub fn http_url_to_ws_rpc(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    let as_ws = if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        trimmed.to_string()
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        format!("ws://{trimmed}")
    };
    crate::discovery::with_ws_rpc_path(&as_ws)
}
