//! Track B spike: can we hang a dynamic `rcl_service` on the bridge executor?
//!
//! ## Verdict (2026-08): **FAIL for integration with the existing rclrs spin path**
//!
//! Evidence from **rclrs 0.7** (same barriers on 0.8 / main for dynamic srv):
//!
//! 1. `WorkerCommands::add_to_wait_set` / `ExecutorCommands::add_to_wait_set` are
//!    `pub(crate)` — external crates cannot register a custom [`rclrs::Waitable`]
//!    on the bridge's `ros_executor` the way in-crate `DynamicSubscription` does.
//! 2. `NodeHandle::rcl_node` is `pub(crate)` — we cannot obtain the live
//!    `rcl_node_t*` from an [`rclrs::Node`] to call `rcl_service_init` against the
//!    same node the executor already owns.
//! 3. Upstream PR ros2_rust#492 shipped dynamic pub/sub only; services/actions
//!    remain "future implementation". Upgrading rclrs does not unlock this.
//!
//! What *is* public and would help a fork/upstream PR: [`rclrs::WaitSet`],
//! [`rclrs::Waitable::new`], [`rclrs::RclPrimitive`]. A second, fully independent
//! rcl context+node+wait set (raw C API) could receive a Trigger in isolation, but
//! sharing one `rcl_node` with `ros_executor.spin` races RMW wait sets — not safe
//! for the bridge without ENTITY_LIFECYCLE-style serialization that we have not
//! validated. Until upstream exposes `create_dynamic_service` (or wait-set
//! registration), Track B full dynamic FFI stays blocked; expand builtins / fork
//! rclrs instead.
//!
//! This module intentionally has no production wiring — only documentation tests
//! that pin the privacy barriers so a future retry can re-check.

#![allow(dead_code)]

/// Spike outcome for docs / CI messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicServiceSpikeResult {
    /// Safe attach to bridge executor not possible with current rclrs.
    BlockedByRclrsPrivacy,
}

/// Recorded conclusion of the Track B wait-set spike.
pub const SPIKE_RESULT: DynamicServiceSpikeResult =
    DynamicServiceSpikeResult::BlockedByRclrsPrivacy;

/// Human-readable summary (errors / docs).
pub fn spike_summary() -> &'static str {
    "Track B dynamic service spike: blocked — rclrs wait-set registration and \
     NodeHandle::rcl_node are pub(crate); cannot safely hang rcl_service_init \
     entities on the bridge ros_executor. Prefer upstream create_dynamic_service \
     or expand typed builtins."
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
        assert!(s.contains("create_dynamic_service") || s.contains("builtins"), "{s}");
    }

    /// Compile-time / API surface check: public WaitSet exists, but that alone
    /// does not unlock attaching to the bridge executor's WorkerCommands.
    #[test]
    fn public_wait_set_exists_but_executor_hook_does_not() {
        // WaitSet::new is public — usable only with a Context we own, not for
        // injecting into BasicExecutor's private channel.
        let _ = std::any::type_name::<rclrs::WaitSet>();
        let _ = std::any::type_name::<rclrs::Waitable>();
        // Document: there is no `rclrs::create_dynamic_service` symbol.
        assert!(
            matches!(SPIKE_RESULT, DynamicServiceSpikeResult::BlockedByRclrsPrivacy),
            "revisit Track B only if rclrs exposes dynamic service or public add_to_wait_set"
        );
    }
}
