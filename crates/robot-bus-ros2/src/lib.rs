//! In-process ROS 2 ↔ robot-bus topic/service/action bridge (the `robot-bus-ros2` crate).
//!
//! Requires a sourced ROS 2 distribution so `rclrs` can link against `rcl`.
//! Builtins only use interface packages shipped with `ros_core` / `ros_base`.
//! The `demo` feature adds the `example_interfaces` Fibonacci mapper and the
//! examples that need `ros-<distro>-example-interfaces`; it is off by default.
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

mod drop_stats;
mod lazy_subscribe;

mod builder;
mod deadline;
/// Recorded outcome of the Rust bridge dynamic service/action experiment.
pub mod dynamic_rpc;
mod mapper;
mod observe;
mod spin;
mod typed_rpc;
mod typed_wire;
/// Typed ROS IDL bindings (std_srvs) for the bridge.
pub mod vendor;

pub mod mappers;

/// Library-owned typed service/action attach helpers.
pub mod typed_service {
    #[cfg(feature = "demo")]
    pub use super::typed_rpc::attach_fibonacci;
    pub use super::typed_rpc::{
        attach_builtin_action, attach_builtin_service, attach_set_bool, attach_trigger,
    };
    pub use super::typed_wire::{
        attach_typed_bus_to_ros, create_typed_ros2_to_bus_sub, wire_typed_action,
        wire_typed_service,
    };
}

pub use builder::{
    ACTION_CALL_TIMEOUT, Action, ActionFromBus, ActionFromRos, ActionPair, ActionReady,
    BusToRos2Ready, FromBus, FromBusToRos, FromRos, FromRosToBus, IntoActionMapper,
    IntoServiceMapper, IntoTopicMapper, Ros2Bridge, Ros2BridgeBuilder, Ros2ToBusReady,
    SERVICE_CALL_TIMEOUT, Service, ServiceFromBus, ServiceFromRos, ServicePair, ServiceReady,
};
pub use drop_stats::{DropStats, DropStatsSnapshot, RouteHealth, RpcStats};
pub use lazy_subscribe::{CONSOLE_DETECT_TIMEOUT, should_enable_ros_subscription};
pub use mapper::{
    ActionMapper, ActionWireContext, Direction, ServiceMapper, ServiceWireContext, TopicDurability,
    TopicMapper, TopicQos, TopicQosKeepLast, TopicReliability, TopicWireContext, TypedActionMapper,
    TypedServiceMapper, TypedTopicMapper, lookup_topic_mapper, lookup_topic_mapper_arc,
    registered_topic_types,
};
#[cfg(feature = "demo")]
pub use mappers::action_bridges::FibonacciActionMapper;
pub use mappers::service_bridges::{SetBoolServiceMapper, TriggerServiceMapper};

// Common builtins re-exported for short imports in examples.
pub use mappers::sensor_msgs::image::SensorMsgsImageMapper;
pub use mappers::std_msgs::string::StdMsgsStringMapper;
