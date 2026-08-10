//! In-process ROS 2 ↔ robot-bus topic/service/action bridge (`feature = "ros2"`).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Everyday robot-bus builds leave this feature off.
//!
//! Attach a [`TopicMapper`] / [`ServiceMapper`] / [`ActionMapper`] on each route with
//! `.mapper(...)`. Service/action mappers are **type codecs** (builtin ZST tags);
//! the library owns typed ROS client/server wiring for known builtins. Arbitrary
//! custom service/action codecs need dynamic RPC support (Track B) or a Rust
//! `attach` override.
//!
//! Builtin mapper types live under [`mappers`] (e.g.
//! [`mappers::std_msgs::string::StdMsgsStringMapper`]); YAML / C++ still resolve
//! builtins by type-name string via `lookup_*`.

mod builder;
mod dynamic_rpc_spike;
mod mapper;
mod spin;
mod typed_rpc;
/// Typed ROS IDL bindings (std_srvs / example_interfaces) for the bridge.
pub mod vendor;
mod yaml;

pub mod mappers;

/// DynamicMessage field helpers for implementing custom [`TopicMapper`]s.
pub mod mapper_support {
    pub use super::mappers::common::*;
}

/// Library-owned typed service/action attach helpers (builtins + advanced Rust backends).
pub mod typed_service {
    pub use super::typed_rpc::{
        attach_builtin_action, attach_builtin_service, attach_fibonacci, attach_set_bool,
        attach_trigger,
    };
}

/// Track B spike outcome (dynamic service wait-set feasibility).
pub mod dynamic_rpc {
    pub use super::dynamic_rpc_spike::{
        DynamicServiceSpikeResult, SPIKE_RESULT, spike_summary,
    };
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
