//! Lightweight per-service counters for the service bus.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Counters for a single service name.
#[derive(Debug, Default)]
pub struct ServiceCounters {
    pub calls: AtomicU64,
    pub errors: AtomicU64,
    pub worker_died: AtomicU64,
    pub pending_timeout: AtomicU64,
    pub workers: AtomicU64,
    /// EWMA latency in milliseconds (scaled integer).
    pub latency_ewma_ms: AtomicU64,
    pub last_seen_unix_ms: AtomicU64,
}

/// Snapshot of one service.
#[derive(Clone, Debug)]
pub struct ServiceSnapshot {
    pub name: String,
    pub calls: u64,
    pub errors: u64,
    pub worker_died: u64,
    pub pending_timeout: u64,
    pub workers: u64,
    pub avg_latency_ms: u64,
    pub last_seen_unix_ms: u64,
}

/// Aggregate snapshot.
#[derive(Clone, Debug, Default)]
pub struct ServiceMetricsSnapshot {
    pub total_calls: u64,
    pub services: Vec<ServiceSnapshot>,
}

/// Shared service-bus metrics (console / observers).
#[derive(Debug, Default)]
pub struct ServiceMetrics {
    services: Mutex<HashMap<String, Arc<ServiceCounters>>>,
    /// In-flight call start times keyed by `client_id\\0req_id`.
    pending: Mutex<HashMap<Vec<u8>, Instant>>,
}

impl ServiceMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn counters(&self, name: &str) -> Arc<ServiceCounters> {
        let mut map = self.services.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = map.get(name) {
            existing.clone()
        } else {
            let c = Arc::new(ServiceCounters::default());
            map.insert(name.to_string(), c.clone());
            c
        }
    }

    fn touch(counters: &ServiceCounters) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        counters.last_seen_unix_ms.store(now_ms, Ordering::Relaxed);
    }

    fn pending_key(client_id: &[u8], req_id: &[u8]) -> Vec<u8> {
        let mut k = Vec::with_capacity(client_id.len() + 1 + req_id.len());
        k.extend_from_slice(client_id);
        k.push(0);
        k.extend_from_slice(req_id);
        k
    }

    /// Ensure the service appears in snapshots (e.g. on READY) and refresh worker gauge.
    pub fn set_workers(&self, name: &str, workers: u64) {
        let c = self.counters(name);
        c.workers.store(workers, Ordering::Relaxed);
    }

    /// Successful forward to a worker — start latency timer.
    pub fn record_call_start(&self, name: &str, client_id: &[u8], req_id: &[u8]) {
        let c = self.counters(name);
        c.calls.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        // Cap pending map to avoid unbounded growth on lost replies.
        if pending.len() > 10_000 {
            pending.clear();
        }
        pending.insert(Self::pending_key(client_id, req_id), Instant::now());
    }

    /// Reply delivered — update EWMA latency.
    pub fn record_call_ok(&self, name: &str, client_id: &[u8], req_id: &[u8]) {
        let c = self.counters(name);
        Self::touch(&c);
        let started = {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            pending.remove(&Self::pending_key(client_id, req_id))
        };
        if let Some(started) = started {
            let sample_ms = started.elapsed().as_millis() as u64;
            // EWMA: new = (old * 7 + sample) / 8
            let old = c.latency_ewma_ms.load(Ordering::Relaxed);
            let next = if old == 0 {
                sample_ms
            } else {
                (old.saturating_mul(7).saturating_add(sample_ms)) / 8
            };
            c.latency_ewma_ms.store(next, Ordering::Relaxed);
        }
    }

    /// Broker-synthesized failure (NO_WORKER, etc.).
    pub fn record_error(&self, name: &str) {
        let c = self.counters(name);
        c.errors.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
    }

    pub fn record_worker_died(&self, name: &str) {
        let c = self.counters(name);
        c.worker_died.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
    }

    pub fn record_pending_timeout(&self, name: &str) {
        let c = self.counters(name);
        c.pending_timeout.fetch_add(1, Ordering::Relaxed);
        Self::touch(&c);
    }

    pub fn snapshot(&self) -> ServiceMetricsSnapshot {
        let map = self.services.lock().unwrap_or_else(|e| e.into_inner());
        let mut services: Vec<ServiceSnapshot> = map
            .iter()
            .map(|(name, c)| ServiceSnapshot {
                name: name.clone(),
                calls: c.calls.load(Ordering::Relaxed),
                errors: c.errors.load(Ordering::Relaxed),
                worker_died: c.worker_died.load(Ordering::Relaxed),
                pending_timeout: c.pending_timeout.load(Ordering::Relaxed),
                workers: c.workers.load(Ordering::Relaxed),
                avg_latency_ms: c.latency_ewma_ms.load(Ordering::Relaxed),
                last_seen_unix_ms: c.last_seen_unix_ms.load(Ordering::Relaxed),
            })
            .collect();
        services.sort_by(|a, b| a.name.cmp(&b.name));
        let total_calls = services.iter().map(|s| s.calls).sum();
        ServiceMetricsSnapshot {
            total_calls,
            services,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_ok_updates_ewma() {
        let m = ServiceMetrics::new();
        m.record_call_start("svc.a", b"c1", b"r1");
        m.record_call_ok("svc.a", b"c1", b"r1");
        let s = m.snapshot();
        assert_eq!(s.services.len(), 1);
        assert_eq!(s.services[0].calls, 1);
        assert_eq!(s.services[0].errors, 0);
        assert_eq!(s.total_calls, 1);
    }

    #[test]
    fn error_increments() {
        let m = ServiceMetrics::new();
        m.record_error("svc.b");
        let s = m.snapshot();
        assert_eq!(s.services[0].errors, 1);
    }
}
