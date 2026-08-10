//! In-process ROS 2 ↔ robot-bus topic/service/action bridge (`feature = "ros2"`).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Everyday robot-bus builds leave this feature off.
//!
//! Topic routes are configured by ROS type string and resolved through the
//! [`codec`] topic registry (not a hardcoded enum). Add a type by implementing
//! [`TopicCodec`] and registering it in the builtin table.

mod builder;
mod codec;
mod convert;
mod echo;
mod spin;
mod vendor;
mod yaml;

pub use builder::{
    ACTION_CALL_TIMEOUT, ActKind, ActionRouteBuilder, Direction, Ros2Bridge, Ros2BridgeBuilder,
    RouteBuilder, SERVICE_CALL_TIMEOUT, ServiceRouteBuilder, SrvKind,
};
pub use codec::{TopicCodec, lookup_topic_codec, registered_topic_types};
pub use echo::EchoFilter;
