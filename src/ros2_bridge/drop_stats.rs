//! Per-bridge counters for dropped topic samples (convert / decode / publish).

use std::sync::atomic::{AtomicU64, Ordering};

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
}
