//! Per-bridge drop counters and per-route health (console snapshots / idle).

use crate::errors::BusError;
use std::sync::Mutex;
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
    pub(crate) latched: AtomicBool,
    rpc: Mutex<RpcStats>,
}

#[derive(Debug, Default, Clone)]
pub struct RpcStats {
    pub calls: u64,
    pub failures: u64,
    pub timeouts: u64,
    pub cancelled: u64,
    pub rejected: u64,
    pub last_error: String,
    pub last_status: String,
}

impl RouteHealth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_rx(&self) {
        self.rx.fetch_add(1, Ordering::Relaxed);
        self.last_rx_ms.store(unix_ms(), Ordering::Relaxed);
        self.idle_latched.store(false, Ordering::Relaxed);
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

    pub fn rpc_start(&self) {
        self.rpc.lock().unwrap_or_else(|e| e.into_inner()).calls += 1;
        self.record_rx();
    }

    pub fn rpc_finish(&self, error: Option<&BusError>) {
        let mut stats = self.rpc.lock().unwrap_or_else(|e| e.into_inner());
        let status = match error {
            None => {
                self.record_tx();
                "succeeded"
            }
            Some(BusError::Cancelled { .. }) => {
                stats.cancelled += 1;
                "cancelled"
            }
            Some(err) => {
                stats.failures += 1;
                match err {
                    BusError::Timeout(_) => {
                        stats.timeouts += 1;
                        "timeout"
                    }
                    BusError::ActionRejected(_) => {
                        stats.rejected += 1;
                        "rejected"
                    }
                    BusError::ActionAborted(_) => "aborted",
                    _ => "failed",
                }
            }
        };
        stats.last_status = status.into();
        if let Some(error) = error {
            stats.last_error = error.to_string().chars().take(512).collect();
            if self.should_log_warn() {
                log::warn!("ROS bridge RPC {status}: {error}");
            }
        }
    }

    pub fn rpc_snapshot(&self) -> RpcStats {
        self.rpc.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn is_idle(&self, enabled: bool, grace_elapsed: bool) -> bool {
        let last = self.last_rx_ms.load(Ordering::Relaxed);
        enabled
            && grace_elapsed
            && (last == 0
                || (!self.latched.load(Ordering::Relaxed)
                    && unix_ms().saturating_sub(last) >= 15_000))
    }

    /// One event per idle episode; receiving traffic rearms the warning.
    pub fn take_idle_event(&self, enabled: bool, grace_elapsed: bool) -> bool {
        if !self.is_idle(enabled, grace_elapsed) {
            self.idle_latched.store(false, Ordering::Relaxed);
            return false;
        }
        !self.idle_latched.swap(true, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stall_after_traffic_and_rearms_after_recovery() {
        let h = RouteHealth::new();
        h.record_rx();
        assert!(!h.is_idle(true, true));
        h.last_rx_ms.store(unix_ms() - 16_000, Ordering::Relaxed);
        assert!(h.take_idle_event(true, true));
        assert!(!h.take_idle_event(true, true));
        h.record_rx();
        h.last_rx_ms.store(unix_ms() - 16_000, Ordering::Relaxed);
        assert!(h.take_idle_event(true, true));
        assert!(!h.is_idle(false, true));
        h.latched.store(true, Ordering::Relaxed);
        assert!(!h.is_idle(true, true));
    }

    #[test]
    fn rpc_counters_separate_terminal_outcomes() {
        let h = RouteHealth::new();
        h.rpc_start();
        h.rpc_finish(None);
        for error in [
            BusError::Timeout("late".into()),
            BusError::ActionRejected("rejected".into()),
            BusError::ActionAborted("aborted".into()),
            BusError::Cancelled {
                name: "cancelled".into(),
            },
        ] {
            h.rpc_start();
            h.rpc_finish(Some(&error));
        }
        let stats = h.rpc_snapshot();
        assert_eq!(
            (
                stats.calls,
                stats.failures,
                stats.timeouts,
                stats.rejected,
                stats.cancelled
            ),
            (5, 3, 1, 1, 1)
        );
        assert_eq!((h.rx(), h.tx()), (5, 1));
        assert_eq!(stats.last_status, "cancelled");
    }

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
