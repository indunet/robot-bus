//! In-process ROS 2 ↔ robot-bus topic/service bridge (`feature = "ros2"`).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Everyday robot-bus builds leave this feature off.

mod builder;
mod convert;
mod echo;
mod spin;
mod vendor;
mod yaml;

pub use builder::{
    Direction, MsgKind, Ros2Bridge, Ros2BridgeBuilder, RouteBuilder, ServiceRouteBuilder, SrvKind,
    SERVICE_CALL_TIMEOUT,
};
pub use echo::EchoFilter;
