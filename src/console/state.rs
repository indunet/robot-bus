//! Shared console HTTP state (binds, metrics, event log).

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::broadcast;

use crate::broker::action_bus::{ActionMetrics, ActionMetricsSnapshot};
use crate::broker::message_bus::{MessageMetrics, MessageMetricsSnapshot};
use crate::broker::service_bus::{ServiceMetrics, ServiceMetricsSnapshot};
use crate::tank::TankManager;

use super::topic_registry::TopicTypeRegistry;
use super::topology_registry::TopologyRegistry;

const EVENT_RING_CAP: usize = 500;
const EVENT_BROADCAST_CAP: usize = 64;
const RATE_BASELINE_MS: u64 = 500;
/// Long window so 0.2–0.5 Hz topics show a stable fractional rate.
const RATE_WINDOW: Duration = Duration::from_secs(10);
/// Instantaneous rates at or above this use the ~1 s sample (responsive).
const RATE_SHORT_HZ: f64 = 2.0;

/// Listen / bind addresses shown in the console.
#[derive(Clone, Debug)]
pub struct BrokerEndpoints {
    pub msg_xsub: String,
    pub msg_xpub: String,
    pub svc_fe: String,
    pub svc_be: String,
    pub act_fe: String,
    pub act_be: String,
    pub ws: String,
    pub web: String,
    /// Broker id + connectable endpoints for `GET /api/v1/discover`.
    pub discover: crate::discovery::DiscoverResponse,
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
    history: VecDeque<(Instant, MessageMetricsSnapshot)>,
    view: RateView,
}

#[derive(Clone, Debug)]
struct SvcRateSample {
    history: VecDeque<(Instant, ServiceMetricsSnapshot)>,
    view: ServiceRateView,
}

#[derive(Clone, Debug)]
struct ActRateSample {
    history: VecDeque<(Instant, ActionMetricsSnapshot)>,
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
    /// Lazy tank singleton (started on first console TANK session).
    pub tank: Arc<TankManager>,
    /// When false, tank UI/API acquire is disabled (`--no-tank`).
    pub tank_enabled: bool,
    /// When false, the docs sidebar entry is hidden (`--no-docs`).
    pub docs_enabled: bool,
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
        tank: Arc<TankManager>,
        tank_enabled: bool,
        docs_enabled: bool,
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
            tank,
            tank_enabled,
            docs_enabled,
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
        if !state.endpoints.ws.is_empty() {
            state.events.emit(
                "INFO",
                "ws",
                format!("WebSocket RPC (/ws-rpc) gateway at {}", state.endpoints.ws),
            );
        }
        if !state.tank_enabled {
            state.events.emit(
                "INFO",
                "tank",
                "tank demo disabled (--no-tank); console menu hidden",
            );
        }
        if !state.docs_enabled {
            state.events.emit(
                "INFO",
                "docs",
                "docs sidebar disabled (--no-docs); console menu hidden",
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

    /// Compute message rates from a ~1 s sample blended with a 10 s window.
    ///
    /// Baseline advances at most once per ~500ms so concurrent `/status` + `/topics`
    /// reads share a stable window.
    pub fn rates(&self) -> RateView {
        let now = Instant::now();
        let mut guard = self.msg_rate.lock().unwrap_or_else(|e| e.into_inner());
        let snap = self.metrics.snapshot();

        if reuse_view(
            guard
                .as_ref()
                .and_then(|s| s.history.back().map(|(at, _)| *at)),
            now,
        ) {
            return guard.as_ref().expect("reuse after Some").view.clone();
        }

        let sample = guard.get_or_insert_with(|| MsgRateSample {
            history: VecDeque::new(),
            view: rate_view_zero(&snap),
        });
        push_history(&mut sample.history, now, snap.clone());
        sample.view = match window_baselines(&sample.history) {
            Some((short_prev, short_dt, long_prev, long_dt)) => {
                rate_view_blended(&snap, short_prev, short_dt, long_prev, long_dt)
            }
            None => rate_view_zero(&snap),
        };
        sample.view.clone()
    }

    /// Service call rates (accepted/forwarded calls per second).
    pub fn service_rates(&self) -> ServiceRateView {
        let now = Instant::now();
        let mut guard = self.svc_rate.lock().unwrap_or_else(|e| e.into_inner());
        let snap = self.service_metrics.snapshot();

        if reuse_view(
            guard
                .as_ref()
                .and_then(|s| s.history.back().map(|(at, _)| *at)),
            now,
        ) && guard.as_ref().is_some_and(|s| {
            s.view.services.len() == snap.services.len()
                && s.view
                    .services
                    .iter()
                    .map(|x| x.name.as_str())
                    .eq(snap.services.iter().map(|x| x.name.as_str()))
        }) {
            return guard.as_ref().expect("reuse after Some").view.clone();
        }

        let sample = guard.get_or_insert_with(|| SvcRateSample {
            history: VecDeque::new(),
            view: service_rate_zero(&snap),
        });
        push_history(&mut sample.history, now, snap.clone());
        sample.view = match window_baselines(&sample.history) {
            Some((short_prev, short_dt, long_prev, long_dt)) => {
                service_rate_blended(&snap, short_prev, short_dt, long_prev, long_dt)
            }
            None => service_rate_zero(&snap),
        };
        sample.view.clone()
    }

    /// Action run rates (accepted goals per second).
    pub fn action_rates(&self) -> ActionRateView {
        let now = Instant::now();
        let mut guard = self.act_rate.lock().unwrap_or_else(|e| e.into_inner());
        let snap = self.action_metrics.snapshot();

        if reuse_view(
            guard
                .as_ref()
                .and_then(|s| s.history.back().map(|(at, _)| *at)),
            now,
        ) && guard.as_ref().is_some_and(|s| {
            s.view.actions.len() == snap.actions.len()
                && s.view
                    .actions
                    .iter()
                    .map(|x| x.name.as_str())
                    .eq(snap.actions.iter().map(|x| x.name.as_str()))
        }) {
            return guard.as_ref().expect("reuse after Some").view.clone();
        }

        let sample = guard.get_or_insert_with(|| ActRateSample {
            history: VecDeque::new(),
            view: action_rate_zero(&snap),
        });
        push_history(&mut sample.history, now, snap.clone());
        sample.view = match window_baselines(&sample.history) {
            Some((short_prev, short_dt, long_prev, long_dt)) => {
                action_rate_blended(&snap, short_prev, short_dt, long_prev, long_dt)
            }
            None => action_rate_zero(&snap),
        };
        sample.view.clone()
    }
}

fn reuse_view(last_at: Option<Instant>, now: Instant) -> bool {
    last_at.is_some_and(|at| now.duration_since(at) < Duration::from_millis(RATE_BASELINE_MS))
}

fn push_history<T>(history: &mut VecDeque<(Instant, T)>, now: Instant, snap: T) {
    history.push_back((now, snap));
    while history.len() > 1 && now.duration_since(history[0].0) > RATE_WINDOW {
        history.pop_front();
    }
}

fn window_baselines<T>(history: &VecDeque<(Instant, T)>) -> Option<(&T, f64, &T, f64)> {
    let n = history.len();
    if n < 2 {
        return None;
    }
    let (now, _) = history.back()?;
    let (short_at, short_prev) = history.get(n - 2)?;
    let (long_at, long_prev) = history.front()?;
    let short_dt = now.duration_since(*short_at).as_secs_f64().max(1e-3);
    let long_dt = now.duration_since(*long_at).as_secs_f64().max(1e-3);
    Some((short_prev, short_dt, long_prev, long_dt))
}

/// High-rate traffic uses the ~1 s sample; sub-2 Hz uses the 10 s window.
pub(crate) fn pick_rate(short: f64, long: f64) -> f64 {
    if short >= RATE_SHORT_HZ { short } else { long }
}

pub(crate) fn quantize_hz(r: f64) -> f64 {
    if !r.is_finite() || r <= 0.0 {
        0.0
    } else {
        (r * 1000.0).round() / 1000.0
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

fn rate_view_blended(
    snap: &MessageMetricsSnapshot,
    short_prev: &MessageMetricsSnapshot,
    short_dt: f64,
    long_prev: &MessageMetricsSnapshot,
    long_dt: f64,
) -> RateView {
    let short = rate_view_from_delta(snap, short_prev, short_dt);
    let long = rate_view_from_delta(snap, long_prev, long_dt);
    let long_by: HashMap<&str, (f64, f64)> = long
        .topics
        .iter()
        .map(|t| (t.name.as_str(), (t.msg_per_sec, t.bytes_per_sec)))
        .collect();
    let topics = short
        .topics
        .into_iter()
        .map(|mut t| {
            let (long_hz, long_bytes) = long_by.get(t.name.as_str()).copied().unwrap_or((0.0, 0.0));
            let use_short = t.msg_per_sec >= RATE_SHORT_HZ;
            if !use_short {
                t.msg_per_sec = long_hz;
                t.bytes_per_sec = long_bytes;
            }
            t
        })
        .collect();
    RateView {
        msg_per_sec: pick_rate(short.msg_per_sec, long.msg_per_sec),
        bytes_per_sec: if short.msg_per_sec >= RATE_SHORT_HZ {
            short.bytes_per_sec
        } else {
            long.bytes_per_sec
        },
        total_msgs: short.total_msgs,
        total_bytes: short.total_bytes,
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

fn service_rate_blended(
    snap: &ServiceMetricsSnapshot,
    short_prev: &ServiceMetricsSnapshot,
    short_dt: f64,
    long_prev: &ServiceMetricsSnapshot,
    long_dt: f64,
) -> ServiceRateView {
    let short = service_rate_from_delta(snap, short_prev, short_dt);
    let long = service_rate_from_delta(snap, long_prev, long_dt);
    let long_by: HashMap<&str, f64> = long
        .services
        .iter()
        .map(|s| (s.name.as_str(), s.calls_per_sec))
        .collect();
    let services = short
        .services
        .into_iter()
        .map(|mut s| {
            let long_hz = long_by.get(s.name.as_str()).copied().unwrap_or(0.0);
            s.calls_per_sec = pick_rate(s.calls_per_sec, long_hz);
            s
        })
        .collect();
    ServiceRateView {
        calls_per_sec: pick_rate(short.calls_per_sec, long.calls_per_sec),
        total_calls: short.total_calls,
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
                workers: a.workers,
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
            workers: a.workers,
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

fn action_rate_blended(
    snap: &ActionMetricsSnapshot,
    short_prev: &ActionMetricsSnapshot,
    short_dt: f64,
    long_prev: &ActionMetricsSnapshot,
    long_dt: f64,
) -> ActionRateView {
    let short = action_rate_from_delta(snap, short_prev, short_dt);
    let long = action_rate_from_delta(snap, long_prev, long_dt);
    let long_by: HashMap<&str, f64> = long
        .actions
        .iter()
        .map(|a| (a.name.as_str(), a.runs_per_sec))
        .collect();
    let actions = short
        .actions
        .into_iter()
        .map(|mut a| {
            let long_hz = long_by.get(a.name.as_str()).copied().unwrap_or(0.0);
            a.runs_per_sec = pick_rate(a.runs_per_sec, long_hz);
            a
        })
        .collect();
    ActionRateView {
        runs_per_sec: pick_rate(short.runs_per_sec, long.runs_per_sec),
        total_runs: short.total_runs,
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
    pub workers: u64,
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

#[cfg(test)]
mod tests {
    use super::{pick_rate, quantize_hz};

    #[test]
    fn pick_rate_keeps_short_when_busy() {
        assert!((pick_rate(50.0, 40.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pick_rate_uses_long_window_below_2hz() {
        assert!((pick_rate(1.0, 0.2) - 0.2).abs() < f64::EPSILON);
        assert!((pick_rate(0.0, 0.5) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn quantize_hz_three_decimals() {
        assert_eq!(quantize_hz(0.2), 0.2);
        assert_eq!(quantize_hz(0.0), 0.0);
        assert!((quantize_hz(1.23456) - 1.235).abs() < 1e-9);
    }
}
