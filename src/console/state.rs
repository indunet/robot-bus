//! Shared console HTTP state (binds, metrics, event log).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::broadcast;

use crate::bot_sim::BotSimManager;
use crate::broker::action_bus::{ActionMetrics, ActionMetricsSnapshot};
use crate::broker::message_bus::{MessageMetrics, MessageMetricsSnapshot};
use crate::broker::service_bus::{ServiceMetrics, ServiceMetricsSnapshot};

use super::topic_registry::TopicTypeRegistry;
use super::topology_registry::TopologyRegistry;

const EVENT_RING_CAP: usize = 500;
const EVENT_BROADCAST_CAP: usize = 64;
const RATE_BASELINE_MS: u64 = 500;

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
struct MsgRateSample {
    at: Instant,
    snap: MessageMetricsSnapshot,
    view: RateView,
}

#[derive(Clone, Debug)]
struct SvcRateSample {
    at: Instant,
    snap: ServiceMetricsSnapshot,
    view: ServiceRateView,
}

#[derive(Clone, Debug)]
struct ActRateSample {
    at: Instant,
    snap: ActionMetricsSnapshot,
    view: ActionRateView,
}

/// Process-wide console state shared with Axum handlers.
pub struct ConsoleState {
    pub started_at: Instant,
    pub version: &'static str,
    pub pid: u32,
    pub endpoints: BrokerEndpoints,
    pub metrics: Arc<MessageMetrics>,
    pub service_metrics: Arc<ServiceMetrics>,
    pub action_metrics: Arc<ActionMetrics>,
    pub topic_types: Arc<TopicTypeRegistry>,
    pub topology: Arc<TopologyRegistry>,
    pub events: EventLog,
    /// Lazy bot_sim singleton (started on first console BOT SIM session).
    pub bot_sim: Arc<BotSimManager>,
    msg_rate: Mutex<Option<MsgRateSample>>,
    svc_rate: Mutex<Option<SvcRateSample>>,
    act_rate: Mutex<Option<ActRateSample>>,
}

impl ConsoleState {
    pub fn new(
        endpoints: BrokerEndpoints,
        metrics: Arc<MessageMetrics>,
        service_metrics: Arc<ServiceMetrics>,
        action_metrics: Arc<ActionMetrics>,
        bot_sim: Arc<BotSimManager>,
    ) -> Arc<Self> {
        let state = Arc::new(Self {
            started_at: Instant::now(),
            version: env!("CARGO_PKG_VERSION"),
            pid: std::process::id(),
            endpoints,
            metrics,
            service_metrics,
            action_metrics,
            topic_types: TopicTypeRegistry::new(),
            topology: TopologyRegistry::new(),
            events: EventLog::new(),
            bot_sim,
            msg_rate: Mutex::new(None),
            svc_rate: Mutex::new(None),
            act_rate: Mutex::new(None),
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

    pub fn services_snapshot(&self) -> ServiceMetricsSnapshot {
        self.service_metrics.snapshot()
    }

    pub fn actions_snapshot(&self) -> ActionMetricsSnapshot {
        self.action_metrics.snapshot()
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Compute message rates from the delta since the last baseline sample.
    ///
    /// Baseline advances at most once per ~500ms so concurrent `/status` + `/topics`
    /// reads share a stable window.
    pub fn rates(&self) -> RateView {
        let now = Instant::now();
        let mut guard = self.msg_rate.lock().unwrap_or_else(|e| e.into_inner());
        let snap = self.metrics.snapshot();

        if let Some(prev) = guard.as_ref()
            && now.duration_since(prev.at) < Duration::from_millis(RATE_BASELINE_MS)
            && snap.total_msgs == prev.snap.total_msgs
            && snap.topics.len() == prev.snap.topics.len()
        {
            return prev.view.clone();
        }

        let view = match guard.as_ref() {
            Some(prev) => {
                let dt = now.duration_since(prev.at).as_secs_f64().max(1e-3);
                rate_view_from_delta(&snap, &prev.snap, dt)
            }
            None => rate_view_zero(&snap),
        };

        *guard = Some(MsgRateSample {
            at: now,
            snap,
            view: view.clone(),
        });
        view
    }

    /// Service call rates (accepted/forwarded calls per second).
    pub fn service_rates(&self) -> ServiceRateView {
        let now = Instant::now();
        let mut guard = self.svc_rate.lock().unwrap_or_else(|e| e.into_inner());
        let snap = self.service_metrics.snapshot();

        if let Some(prev) = guard.as_ref()
            && now.duration_since(prev.at) < Duration::from_millis(RATE_BASELINE_MS)
            && snap.total_calls == prev.snap.total_calls
            && snap.services.len() == prev.snap.services.len()
        {
            return prev.view.clone();
        }

        let view = match guard.as_ref() {
            Some(prev) => {
                let dt = now.duration_since(prev.at).as_secs_f64().max(1e-3);
                service_rate_from_delta(&snap, &prev.snap, dt)
            }
            None => service_rate_zero(&snap),
        };

        *guard = Some(SvcRateSample {
            at: now,
            snap,
            view: view.clone(),
        });
        view
    }

    /// Action run rates (accepted goals per second).
    pub fn action_rates(&self) -> ActionRateView {
        let now = Instant::now();
        let mut guard = self.act_rate.lock().unwrap_or_else(|e| e.into_inner());
        let snap = self.action_metrics.snapshot();

        if let Some(prev) = guard.as_ref()
            && now.duration_since(prev.at) < Duration::from_millis(RATE_BASELINE_MS)
            && snap.total_runs == prev.snap.total_runs
            && snap.actions.len() == prev.snap.actions.len()
        {
            return prev.view.clone();
        }

        let view = match guard.as_ref() {
            Some(prev) => {
                let dt = now.duration_since(prev.at).as_secs_f64().max(1e-3);
                action_rate_from_delta(&snap, &prev.snap, dt)
            }
            None => action_rate_zero(&snap),
        };

        *guard = Some(ActRateSample {
            at: now,
            snap,
            view: view.clone(),
        });
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

fn service_rate_zero(snap: &ServiceMetricsSnapshot) -> ServiceRateView {
    ServiceRateView {
        calls_per_sec: 0.0,
        total_calls: snap.total_calls,
        services: snap
            .services
            .iter()
            .map(|s| ServiceRate {
                name: s.name.clone(),
                calls: s.calls,
                errors: s.errors,
                workers: s.workers,
                avg_latency_ms: s.avg_latency_ms,
                last_seen_unix_ms: s.last_seen_unix_ms,
                calls_per_sec: 0.0,
            })
            .collect(),
    }
}

fn service_rate_from_delta(
    snap: &ServiceMetricsSnapshot,
    prev: &ServiceMetricsSnapshot,
    dt: f64,
) -> ServiceRateView {
    let call_delta = snap.total_calls.saturating_sub(prev.total_calls) as f64;
    let mut services = Vec::with_capacity(snap.services.len());
    for s in &snap.services {
        let prev_s = prev.services.iter().find(|p| p.name == s.name);
        let dc = match prev_s {
            Some(p) => s.calls.saturating_sub(p.calls) as f64,
            None => 0.0,
        };
        services.push(ServiceRate {
            name: s.name.clone(),
            calls: s.calls,
            errors: s.errors,
            workers: s.workers,
            avg_latency_ms: s.avg_latency_ms,
            last_seen_unix_ms: s.last_seen_unix_ms,
            calls_per_sec: dc / dt,
        });
    }
    ServiceRateView {
        calls_per_sec: call_delta / dt,
        total_calls: snap.total_calls,
        services,
    }
}

fn action_rate_zero(snap: &ActionMetricsSnapshot) -> ActionRateView {
    ActionRateView {
        runs_per_sec: 0.0,
        total_runs: snap.total_runs,
        actions: snap
            .actions
            .iter()
            .map(|a| ActionRate {
                name: a.name.clone(),
                runs: a.runs,
                errors: a.errors,
                active: a.active,
                avg_duration_ms: a.avg_duration_ms,
                last_seen_unix_ms: a.last_seen_unix_ms,
                runs_per_sec: 0.0,
            })
            .collect(),
    }
}

fn action_rate_from_delta(
    snap: &ActionMetricsSnapshot,
    prev: &ActionMetricsSnapshot,
    dt: f64,
) -> ActionRateView {
    let run_delta = snap.total_runs.saturating_sub(prev.total_runs) as f64;
    let mut actions = Vec::with_capacity(snap.actions.len());
    for a in &snap.actions {
        let prev_a = prev.actions.iter().find(|p| p.name == a.name);
        let dr = match prev_a {
            Some(p) => a.runs.saturating_sub(p.runs) as f64,
            None => 0.0,
        };
        actions.push(ActionRate {
            name: a.name.clone(),
            runs: a.runs,
            errors: a.errors,
            active: a.active,
            avg_duration_ms: a.avg_duration_ms,
            last_seen_unix_ms: a.last_seen_unix_ms,
            runs_per_sec: dr / dt,
        });
    }
    ActionRateView {
        runs_per_sec: run_delta / dt,
        total_runs: snap.total_runs,
        actions,
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

#[derive(Clone, Debug)]
pub struct ServiceRate {
    pub name: String,
    pub calls: u64,
    pub errors: u64,
    pub workers: u64,
    pub avg_latency_ms: u64,
    pub last_seen_unix_ms: u64,
    pub calls_per_sec: f64,
}

#[derive(Clone, Debug)]
pub struct ServiceRateView {
    pub calls_per_sec: f64,
    pub total_calls: u64,
    pub services: Vec<ServiceRate>,
}

#[derive(Clone, Debug)]
pub struct ActionRate {
    pub name: String,
    pub runs: u64,
    pub errors: u64,
    pub active: u64,
    pub avg_duration_ms: u64,
    pub last_seen_unix_ms: u64,
    pub runs_per_sec: f64,
}

#[derive(Clone, Debug)]
pub struct ActionRateView {
    pub runs_per_sec: f64,
    pub total_runs: u64,
    pub actions: Vec<ActionRate>,
}

fn unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}
