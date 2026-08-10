//! In-process ROS 2 ↔ robot-bus topic/service/action bridge (`feature = "ros2"`).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Everyday robot-bus builds leave this feature off.
//!
//! Attach a [`TopicMapper`] / [`ServiceMapper`] / [`ActionMapper`] on each route with
//! `.mapper(...)`. Builtin mapper types live under [`mappers`] (e.g.
//! [`mappers::std_msgs::string::StdMsgsStringMapper`]); YAML / C++ still resolve
//! builtins by type-name string via `lookup_*`.

mod builder;
mod mapper;
mod spin;
mod vendor;
mod yaml;

pub mod mappers;

/// DynamicMessage field helpers for implementing custom [`TopicMapper`]s.
pub mod mapper_support {
    pub use super::mappers::common::*;
}

pub use builder::{
    ACTION_CALL_TIMEOUT, ActionRouteBuilder, IntoActionMapper, IntoServiceMapper, IntoTopicMapper,
    Ros2Bridge, Ros2BridgeBuilder, RouteBuilder, SERVICE_CALL_TIMEOUT, ServiceRouteBuilder,
};
pub use mapper::{
    ActionMapper, ActionWireContext, Direction, ServiceMapper, ServiceWireContext, TopicMapper,
    lookup_topic_mapper, lookup_topic_mapper_arc, registered_topic_types,
};
pub use mappers::action_bridges::{FibonacciActionMapper, lookup_action_mapper};
pub use mappers::service_bridges::{
    SetBoolServiceMapper, TriggerServiceMapper, lookup_service_mapper,
};

// Common builtins re-exported for short imports in examples.
pub use mappers::sensor_msgs::image::SensorMsgsImageMapper;
pub use mappers::std_msgs::string::StdMsgsStringMapper;
