//! Shared console HTTP state (binds, metrics, event log).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::broadcast;

use crate::broker::message_bus::{MessageMetrics, MessageMetricsSnapshot};

const EVENT_RING_CAP: usize = 500;
const EVENT_BROADCAST_CAP: usize = 64;

/// Listen / bind addresses shown in the console.
#[derive(Clone, Debug)]
pub struct BrokerEndpoints {
    pub msg_xsub: String,
    pub msg_xpub: String,
    pub svc_fe: String,
    pub svc_be: String,
    pub act_fe: String,
    pub act_be: String,
    pub grpc: String,
    pub web: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryDto {
    pub id: String,
    pub ts: u64,
    pub level: String,
    pub source: String,
    pub message: String,
}

/// Ring buffer + broadcast for SSE clients.
pub struct EventLog {
    ring: Mutex<VecDeque<LogEntryDto>>,
    next_id: AtomicU64,
    tx: broadcast::Sender<LogEntryDto>,
}

impl EventLog {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(EVENT_BROADCAST_CAP);
        Self {
            ring: Mutex::new(VecDeque::with_capacity(EVENT_RING_CAP)),
            next_id: AtomicU64::new(1),
            tx,
        }
    }

    pub fn emit(&self, level: &str, source: &str, message: impl Into<String>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = LogEntryDto {
            id: format!("evt-{id}"),
            ts: unix_ms(),
            level: level.to_string(),
            source: source.to_string(),
            message: message.into(),
        };
        {
            let mut ring = self.ring.lock().unwrap_or_else(|e| e.into_inner());
            if ring.len() >= EVENT_RING_CAP {
                ring.pop_front();
            }
            ring.push_back(entry.clone());
        }
        let _ = self.tx.send(entry);
    }

    pub fn recent(&self) -> Vec<LogEntryDto> {
        self.ring
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .cloned()
            .collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LogEntryDto> {
        self.tx.subscribe()
    }
}

#[derive(Clone, Debug)]
struct RateSample {
    at: Instant,
    snap: MessageMetricsSnapshot,
}

/// Process-wide console state shared with Axum handlers.
pub struct ConsoleState {
    pub started_at: Instant,
    pub version: &'static str,
    pub pid: u32,
    pub endpoints: BrokerEndpoints,
    pub metrics: Arc<MessageMetrics>,
    pub events: EventLog,
    rate: Mutex<Option<RateSample>>,
}

impl ConsoleState {
    pub fn new(endpoints: BrokerEndpoints, metrics: Arc<MessageMetrics>) -> Arc<Self> {
        let state = Arc::new(Self {
            started_at: Instant::now(),
            version: env!("CARGO_PKG_VERSION"),
            pid: std::process::id(),
            endpoints,
            metrics,
            events: EventLog::new(),
            rate: Mutex::new(None),
        });
        state.events.emit(
            "INFO",
            "broker",
            format!(
                "Console HTTP listening on {} (version {})",
                state.endpoints.web, state.version
            ),
        );
        state.events.emit(
            "INFO",
            "broker",
            format!(
                "Message bus XSUB={} XPUB={}",
                state.endpoints.msg_xsub, state.endpoints.msg_xpub
            ),
        );
        if !state.endpoints.grpc.is_empty() {
            state.events.emit(
                "INFO",
                "grpc-web",
                format!("gRPC gateway at {}", state.endpoints.grpc),
            );
        }
        state
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Compute rates from the delta since the last baseline sample.
    ///
    /// Baseline advances at most once per ~500ms so concurrent `/status` + `/topics`
    /// reads share a stable window.
    pub fn rates(&self) -> RateView {
        let now = Instant::now();
        let snap = self.metrics.snapshot();
        let mut guard = self.rate.lock().unwrap_or_else(|e| e.into_inner());

        let view = match guard.as_ref() {
            Some(prev) => {
                let dt = now.duration_since(prev.at).as_secs_f64().max(1e-3);
                rate_view_from_delta(&snap, &prev.snap, dt)
            }
            None => rate_view_zero(&snap),
        };

        let should_advance = match guard.as_ref() {
            None => true,
            Some(prev) => now.duration_since(prev.at) >= Duration::from_millis(500),
        };
        if should_advance {
            *guard = Some(RateSample { at: now, snap });
        }
        view
    }
}

fn rate_view_zero(snap: &MessageMetricsSnapshot) -> RateView {
    RateView {
        msg_per_sec: 0.0,
        bytes_per_sec: 0.0,
        total_msgs: snap.total_msgs,
        total_bytes: snap.total_bytes,
        topics: snap
            .topics
            .iter()
            .map(|t| TopicRate {
                name: t.name.clone(),
                total_msgs: t.total_msgs,
                total_bytes: t.total_bytes,
                last_seen_unix_ms: t.last_seen_unix_ms,
                msg_per_sec: 0.0,
                bytes_per_sec: 0.0,
            })
            .collect(),
    }
}

fn rate_view_from_delta(
    snap: &MessageMetricsSnapshot,
    prev: &MessageMetricsSnapshot,
    dt: f64,
) -> RateView {
    let msg_delta = snap.total_msgs.saturating_sub(prev.total_msgs) as f64;
    let byte_delta = snap.total_bytes.saturating_sub(prev.total_bytes) as f64;
    let mut topics = Vec::with_capacity(snap.topics.len());
    for t in &snap.topics {
        let prev_t = prev.topics.iter().find(|p| p.name == t.name);
        let (dm, db) = match prev_t {
            Some(p) => (
                t.total_msgs.saturating_sub(p.total_msgs) as f64,
                t.total_bytes.saturating_sub(p.total_bytes) as f64,
            ),
            None => (0.0, 0.0),
        };
        topics.push(TopicRate {
            name: t.name.clone(),
            total_msgs: t.total_msgs,
            total_bytes: t.total_bytes,
            last_seen_unix_ms: t.last_seen_unix_ms,
            msg_per_sec: dm / dt,
            bytes_per_sec: db / dt,
        });
    }
    RateView {
        msg_per_sec: msg_delta / dt,
        bytes_per_sec: byte_delta / dt,
        total_msgs: snap.total_msgs,
        total_bytes: snap.total_bytes,
        topics,
    }
}

#[derive(Clone, Debug)]
pub struct TopicRate {
    pub name: String,
    pub total_msgs: u64,
    pub total_bytes: u64,
    pub last_seen_unix_ms: u64,
    pub msg_per_sec: f64,
    pub bytes_per_sec: f64,
}

#[derive(Clone, Debug)]
pub struct RateView {
    pub msg_per_sec: f64,
    pub bytes_per_sec: f64,
    pub total_msgs: u64,
    pub total_bytes: u64,
    pub topics: Vec<TopicRate>,
}

fn unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}
