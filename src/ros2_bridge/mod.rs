//! In-process ROS 2 ↔ robot-bus topic/service/action bridge (`feature = "ros2"`).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Everyday robot-bus builds leave this feature off.
//!
//! Configuration is **code-only**: attach a concrete [`TopicMapper`] /
//! [`ServiceMapper`] / [`ActionMapper`] with `.mapper(...)`. There is no YAML
//! loader and no type-name-string route API.
//!
//! Custom service/action codecs implement [`TypedServiceMapper`] /
//! [`TypedActionMapper`] (field convert methods only). Override
//! [`ServiceMapper::attach`] / [`ActionMapper::attach`] only for advanced wiring.
//! String-based dynamic service/action create remains unavailable in rclrs
//! (see [`dynamic_rpc`]).

mod builder;
mod dynamic_rpc_spike;
mod mapper;
mod ros_idl;
mod spin;
mod typed_rpc;
mod typed_wire;
/// Typed ROS IDL bindings (std_srvs) for the bridge.
pub mod vendor;

pub mod mappers;

/// DynamicMessage field helpers for implementing custom [`TopicMapper`]s.
pub mod mapper_support {
    pub use super::mappers::common::*;
}

/// Library-owned typed service/action attach helpers.
pub mod typed_service {
    pub use super::typed_rpc::{
        attach_builtin_action, attach_builtin_service, attach_fibonacci, attach_set_bool,
        attach_trigger,
    };
    pub use super::typed_wire::{wire_typed_action, wire_typed_service};
}

/// Historical note on dynamic service/action feasibility (not a product path).
pub mod dynamic_rpc {
    pub use super::dynamic_rpc_spike::{DynamicServiceSpikeResult, SPIKE_RESULT, spike_summary};
}

pub use builder::{
    ACTION_CALL_TIMEOUT, ActionRouteBuilder, IntoActionMapper, IntoServiceMapper, IntoTopicMapper,
    Ros2Bridge, Ros2BridgeBuilder, RouteBuilder, SERVICE_CALL_TIMEOUT, ServiceRouteBuilder,
};
pub use mapper::{
    ActionMapper, ActionWireContext, Direction, ServiceMapper, ServiceWireContext, TopicMapper,
    TopicRouteQos, TypedActionMapper, TypedServiceMapper, lookup_topic_mapper,
    lookup_topic_mapper_arc, registered_topic_types,
};
pub use mappers::action_bridges::FibonacciActionMapper;
pub use mappers::service_bridges::{SetBoolServiceMapper, TriggerServiceMapper};

// Common builtins re-exported for short imports in examples.
pub use mappers::sensor_msgs::image::SensorMsgsImageMapper;
pub use mappers::std_msgs::string::StdMsgsStringMapper;
