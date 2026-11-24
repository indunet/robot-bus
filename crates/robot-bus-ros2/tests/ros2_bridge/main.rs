//! ROS 2 bridge integration tests.
//!
//! Diagnostics link the bridge crate. The lazy-subscribe and service smoke
//! modules compile with `--features ws` (broker WebSocket + console API).

#[path = "../../../robot-bus/tests/common/mod.rs"]
mod common;

mod ros2_bridge_diagnostics;
#[cfg(feature = "ws")]
mod ros2_bridge_lazy;
#[cfg(feature = "ws")]
mod ros2_bridge_service_smoke;
