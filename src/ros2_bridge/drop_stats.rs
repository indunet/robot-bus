//! Per-bridge drop counters and per-route health (console snapshots / idle).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Point-in-time copy of [`DropStats`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DropStatsSnapshot {
    pub convert_fail: u64,
    pub decode_fail: u64,
    pub publish_fail: u64,
}

/// Atomic drop counters shared with ROS and bus callbacks.
#[derive(Debug, Default)]
pub struct DropStats {
    convert_fail: AtomicU64,
    decode_fail: AtomicU64,
    publish_fail: AtomicU64,
}

impl DropStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> DropStatsSnapshot {
        DropStatsSnapshot {
            convert_fail: self.convert_fail.load(Ordering::Relaxed),
            decode_fail: self.decode_fail.load(Ordering::Relaxed),
            publish_fail: self.publish_fail.load(Ordering::Relaxed),
        }
    }

    pub fn record_convert_fail(&self) {
        self.convert_fail.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_decode_fail(&self) {
        self.decode_fail.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_publish_fail(&self) {
        self.publish_fail.fetch_add(1, Ordering::Relaxed);
    }
}

/// Unix epoch milliseconds (best-effort; 0 if the clock is before the epoch).
pub fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

const WARN_INTERVAL_MS: u64 = 1000;

/// Per-route counters captured in forward closures (no HashMap lookup).
#[derive(Debug, Default)]
pub struct RouteHealth {
    rx: AtomicU64,
    tx: AtomicU64,
    convert_fail: AtomicU64,
    decode_fail: AtomicU64,
    publish_fail: AtomicU64,
    last_rx_ms: AtomicU64,
    last_warn_ms: AtomicU64,
    idle_latched: AtomicBool,
}

impl RouteHealth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_rx(&self) {
        self.rx.fetch_add(1, Ordering::Relaxed);
        self.last_rx_ms.store(unix_ms(), Ordering::Relaxed);
    }

    pub fn record_tx(&self) {
        self.tx.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_convert_fail(&self) {
        self.convert_fail.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_decode_fail(&self) {
        self.decode_fail.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_publish_fail(&self) {
        self.publish_fail.fetch_add(1, Ordering::Relaxed);
    }

    /// First failure always logs; then at most once per second per route.
    pub fn should_log_warn(&self) -> bool {
        let now = unix_ms();
        let prev = self.last_warn_ms.load(Ordering::Relaxed);
        if prev != 0 && now.saturating_sub(prev) < WARN_INTERVAL_MS {
            return false;
        }
        self.last_warn_ms.store(now, Ordering::Relaxed);
        true
    }

    pub fn rx(&self) -> u64 {
        self.rx.load(Ordering::Relaxed)
    }

    pub fn tx(&self) -> u64 {
        self.tx.load(Ordering::Relaxed)
    }

    pub fn convert_fail(&self) -> u64 {
        self.convert_fail.load(Ordering::Relaxed)
    }

    pub fn decode_fail(&self) -> u64 {
        self.decode_fail.load(Ordering::Relaxed)
    }

    pub fn publish_fail(&self) -> u64 {
        self.publish_fail.load(Ordering::Relaxed)
    }

    pub fn last_rx_ms(&self) -> u64 {
        self.last_rx_ms.load(Ordering::Relaxed)
    }

    pub fn is_idle(&self, enabled: bool, grace_elapsed: bool) -> bool {
        enabled && grace_elapsed && self.last_rx_ms.load(Ordering::Relaxed) == 0
    }

    /// True once when an enabled topic route has never received after grace.
    pub fn take_idle_event(&self, enabled: bool, grace_elapsed: bool) -> bool {
        if self.last_rx_ms.load(Ordering::Relaxed) != 0 {
            self.idle_latched.store(false, Ordering::Relaxed);
            return false;
        }
        if !self.is_idle(enabled, grace_elapsed) {
            return false;
        }
        !self.idle_latched.swap(true, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_starts_zero_and_counts() {
        let stats = DropStats::new();
        assert_eq!(stats.snapshot(), DropStatsSnapshot::default());
        stats.record_convert_fail();
        stats.record_decode_fail();
        stats.record_publish_fail();
        stats.record_publish_fail();
        let snap = stats.snapshot();
        assert_eq!(snap.convert_fail, 1);
        assert_eq!(snap.decode_fail, 1);
        assert_eq!(snap.publish_fail, 2);
    }

    #[test]
    fn route_health_counts_and_idle() {
        let h = RouteHealth::new();
        assert!(!h.take_idle_event(true, false));
        assert!(h.take_idle_event(true, true));
        assert!(!h.take_idle_event(true, true));
        h.record_rx();
        h.record_tx();
        assert!(!h.is_idle(true, true));
        assert!(!h.take_idle_event(true, true));
        assert_eq!(h.rx(), 1);
        assert_eq!(h.tx(), 1);
    }

    #[test]
    fn warn_rate_limit_first_then_silence() {
        let h = RouteHealth::new();
        assert!(h.should_log_warn());
        assert!(!h.should_log_warn());
        h.record_convert_fail();
        assert_eq!(h.convert_fail(), 1);
    }
}
