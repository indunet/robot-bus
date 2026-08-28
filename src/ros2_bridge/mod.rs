//! In-process ROS 2 ↔ robot-bus topic/service/action bridge (`feature = "ros2"`).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Everyday robot-bus builds leave this feature off.
//!
//! Configuration is **code-only**: attach a concrete [`TopicMapper`] /
//! [`ServiceMapper`] / [`ActionMapper`] with `.mapper(...)`. There is no YAML
//! loader and no type-name-string route API.
//!
//! Custom topic codecs implement [`TypedTopicMapper`] (ROS object ↔ protobuf
//! object). Custom service/action codecs implement [`TypedServiceMapper`] /
//! [`TypedActionMapper`]. Override [`TopicMapper::create_ros2_to_bus_subscription`]
//! / [`ServiceMapper::attach`] / [`ActionMapper::attach`] only for advanced wiring.
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

/// Library-owned typed service/action attach helpers.
pub mod typed_service {
    pub use super::typed_rpc::{
        attach_builtin_action, attach_builtin_service, attach_fibonacci, attach_set_bool,
        attach_trigger,
    };
    pub use super::typed_wire::{
        attach_typed_bus_to_ros, create_typed_ros2_to_bus_sub, wire_typed_action,
        wire_typed_service,
    };
}

/// Historical note on dynamic service/action feasibility (not a product path).
pub mod dynamic_rpc {
    pub use super::dynamic_rpc_spike::{spike_summary, DynamicServiceSpikeResult, SPIKE_RESULT};
}

pub use builder::{
    ActionRouteBuilder, IntoActionMapper, IntoServiceMapper, IntoTopicMapper, Ros2Bridge,
    Ros2BridgeBuilder, RouteBuilder, ServiceRouteBuilder, ACTION_CALL_TIMEOUT,
    SERVICE_CALL_TIMEOUT,
};
pub use mapper::{
    lookup_topic_mapper, lookup_topic_mapper_arc, registered_topic_types, ActionMapper,
    ActionWireContext, Direction, ServiceMapper, ServiceWireContext, TopicMapper, TopicRouteQos,
    TopicWireContext, TypedActionMapper, TypedServiceMapper, TypedTopicMapper,
};
pub use mappers::action_bridges::FibonacciActionMapper;
pub use mappers::service_bridges::{SetBoolServiceMapper, TriggerServiceMapper};

// Common builtins re-exported for short imports in examples.
pub use mappers::sensor_msgs::image::SensorMsgsImageMapper;
pub use mappers::std_msgs::string::StdMsgsStringMapper;
