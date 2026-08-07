//! Lightweight per-action counters for the action bus.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Counters for a single action name.
#[derive(Debug, Default)]
pub struct ActionCounters {
    pub runs: AtomicU64,
    pub errors: AtomicU64,
    pub worker_died: AtomicU64,
    pub cancelled: AtomicU64,
    pub active: AtomicU64,
    pub duration_ewma_ms: AtomicU64,
    pub last_seen_unix_ms: AtomicU64,
}

/// Snapshot of one action.
#[derive(Clone, Debug)]
pub struct ActionSnapshot {
    pub name: String,
    pub runs: u64,
    pub errors: u64,
    pub worker_died: u64,
    pub cancelled: u64,
    pub active: u64,
    pub avg_duration_ms: u64,
    pub last_seen_unix_ms: u64,
}

/// Aggregate snapshot.
#[derive(Clone, Debug, Default)]
pub struct ActionMetricsSnapshot {
    pub total_runs: u64,
    pub actions: Vec<ActionSnapshot>,
}

/// Shared action-bus metrics (console / observers).
#[derive(Debug, Default)]
pub struct ActionMetrics {
    actions: Mutex<HashMap<String, Arc<ActionCounters>>>,
    /// In-flight goal start times keyed by goal_id bytes.
    pending: Mutex<HashMap<Vec<u8>, (String, Instant)>>,
}

impl ActionMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn counters(&self, name: &str) -> Arc<ActionCounters> {
        let mut map = self.actions.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = map.get(name) {
            existing.clone()
        } else {
            let c = Arc::new(ActionCounters::default());
            map.insert(name.to_string(), c.clone());
            c
        }
    }

    fn touch(counters: &ActionCounters) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        counters.last_seen_unix_ms.store(now_ms, Ordering::Relaxed);
    }

    /// Ensure the action appears (e.g. on READY).
    pub fn ensure(&self, name: &str) {
        let _ = self.counters(name);
    }

    /// GOAL accepted / forwarded.
    pub fn record_run_start(&self, name: &str, goal_id: &[u8]) {
        let c = self.counters(name);
        c.runs.fetch_add(1, Ordering::Relaxed);
        c.active.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        if pending.len() > 10_000 {
            pending.clear();
        }
        pending.insert(goal_id.to_vec(), (name.to_string(), Instant::now()));
    }

    /// RESULT (success path) — decrement active, update EWMA duration.
    pub fn record_run_ok(&self, name: &str, goal_id: &[u8]) {
        let c = self.counters(name);
        let _ = c
            .active
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(v.saturating_sub(1))
            });
        Self::touch(&c);
        let started = {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            pending.remove(goal_id)
        };
        if let Some((_, started)) = started {
            let sample_ms = started.elapsed().as_millis() as u64;
            let old = c.duration_ewma_ms.load(Ordering::Relaxed);
            let next = if old == 0 {
                sample_ms
            } else {
                (old.saturating_mul(7).saturating_add(sample_ms)) / 8
            };
            c.duration_ewma_ms.store(next, Ordering::Relaxed);
        }
    }

    /// Broker-synthesized failure; if `goal_id` is Some, also clear active/pending.
    pub fn record_error(&self, name: &str, goal_id: Option<&[u8]>) {
        let c = self.counters(name);
        c.errors.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
        if let Some(gid) = goal_id {
            let had = {
                let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
                pending.remove(gid).is_some()
            };
            if had {
                let _ = c
                    .active
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                        Some(v.saturating_sub(1))
                    });
            }
        }
    }

    pub fn record_worker_died(&self, name: &str) {
        let c = self.counters(name);
        c.worker_died.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
    }

    pub fn record_cancelled(&self, name: &str) {
        let c = self.counters(name);
        c.cancelled.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
    }

    pub fn snapshot(&self) -> ActionMetricsSnapshot {
        let map = self.actions.lock().unwrap_or_else(|e| e.into_inner());
        let mut actions: Vec<ActionSnapshot> = map
            .iter()
            .map(|(name, c)| ActionSnapshot {
                name: name.clone(),
                runs: c.runs.load(Ordering::Relaxed),
                errors: c.errors.load(Ordering::Relaxed),
                worker_died: c.worker_died.load(Ordering::Relaxed),
                cancelled: c.cancelled.load(Ordering::Relaxed),
                active: c.active.load(Ordering::Relaxed),
                avg_duration_ms: c.duration_ewma_ms.load(Ordering::Relaxed),
                last_seen_unix_ms: c.last_seen_unix_ms.load(Ordering::Relaxed),
            })
            .collect();
        actions.sort_by(|a, b| a.name.cmp(&b.name));
        let total_runs = actions.iter().map(|a| a.runs).sum();
        ActionMetricsSnapshot {
            total_runs,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_lifecycle() {
        let m = ActionMetrics::new();
        m.record_run_start("act.a", b"g1");
        assert_eq!(m.snapshot().actions[0].active, 1);
        m.record_run_ok("act.a", b"g1");
        let s = m.snapshot().actions[0].clone();
        assert_eq!(s.runs, 1);
        assert_eq!(s.active, 0);
        assert_eq!(s.errors, 0);
        assert_eq!(m.snapshot().total_runs, 1);
    }
}
