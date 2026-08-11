//! Historical note: dynamic ROS service/action on the bridge executor.
//!
//! Ros2Bridge mounts service/action via **typed** rclrs entities (concrete `T`).
//! Dynamic service/action attach via type-name string remains blocked (wait-set
//! registration / `NodeHandle::rcl_node` are `pub(crate)`; upstream dynamic
//! pub/sub only). Prefer per-language native bridges or typed `attach` overrides.

#![allow(dead_code)]

/// Spike outcome for docs / CI messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicServiceSpikeResult {
    /// Safe attach to bridge executor not possible with current rclrs.
    BlockedByRclrsPrivacy,
}

/// Recorded conclusion of the dynamic service wait-set spike.
pub const SPIKE_RESULT: DynamicServiceSpikeResult =
    DynamicServiceSpikeResult::BlockedByRclrsPrivacy;

/// Human-readable summary (errors / docs).
pub fn spike_summary() -> &'static str {
    "Ros2Bridge service/action use typed attach only; dynamic ROS service/action \
     by type-name string is blocked — rclrs wait-set registration and \
     NodeHandle::rcl_node are pub(crate). Prefer native per-language bridges or \
     override ServiceMapper::attach / ActionMapper::attach with a concrete type."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spike_records_blocked_verdict() {
        assert_eq!(
            SPIKE_RESULT,
            DynamicServiceSpikeResult::BlockedByRclrsPrivacy
        );
        let s = spike_summary();
        assert!(s.contains("pub(crate)"), "{s}");
        assert!(s.contains("typed"), "{s}");
    }
}
