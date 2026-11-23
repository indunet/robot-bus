//! ROS 2 bridge integration tests.
//!
//! Diagnostics run without ROS. The lazy-subscribe and service smoke modules
//! compile only with `ws` and `ros2`.

#[path = "../common/mod.rs"]
mod common;

// `drop_stats.rs` is compiled into this binary and imports `crate::errors`.
mod errors {
    pub use robot_bus::errors::BusError;
}

mod ros2_bridge_diagnostics;
#[cfg(all(feature = "ws", feature = "ros2"))]
mod ros2_bridge_lazy;
#[cfg(all(feature = "ws", feature = "ros2"))]
mod ros2_bridge_service_smoke;
