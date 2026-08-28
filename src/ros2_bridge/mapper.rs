//! Topic / service / action mapper traits and topic builtin registry.
//!
//! Topics use [`TypedTopicMapper`] (ROS IDL object ↔ protobuf object).
//! Services / actions use [`TypedServiceMapper`] / [`TypedActionMapper`].
//! The library wires ROS↔bus. Override [`TopicMapper::create_ros2_to_bus_subscription`]
//! / [`TopicMapper::attach_bus_to_ros`] / [`ServiceMapper::attach`] /
//! [`ActionMapper::attach`] only for advanced cases.
//! Route mounting is always via concrete mapper objects (`.mapper(...)`).

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use prost::Message as ProstMessage;
use rclrs::IntoPrimitiveOptions;
use rosidl_runtime_rs::{Action as ActionIdl, Message as RosMessage, Service as ServiceIdl};

use crate::errors::{BusError, Result as BusResult};
use crate::runtime::{Node, TopicPublisherRaw};

use super::mappers::BUILTIN_MAPPER_LIST;
use super::typed_rpc;
use super::typed_wire::{
    attach_typed_bus_to_ros, create_typed_ros2_to_bus_sub, wire_typed_action, wire_typed_service,
};

type Result<T> = std::result::Result<T, BusError>;

/// Topic / service / action bridge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ros2ToBus,
    BusToRos2,
}

/// Reliability for [`TopicQos`]. Must be chosen explicitly (no default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicReliability {
    Reliable,
    BestEffort,
}

/// Intermediate after [`TopicQos::keep_last`]; call [`.reliable()`](TopicQosKeepLast::reliable)
/// or [`.best_effort()`](TopicQosKeepLast::best_effort) to finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicQosKeepLast {
    depth: i32,
}

impl TopicQosKeepLast {
    pub fn reliable(self) -> TopicQos {
        TopicQos {
            depth: self.depth,
            reliability: TopicReliability::Reliable,
        }
    }

    pub fn best_effort(self) -> TopicQos {
        TopicQos {
            depth: self.depth,
            reliability: TopicReliability::BestEffort,
        }
    }
}

/// Bridge QoS: KeepLast depth plus reliability.
///
/// Same type on **ROS** endpoints for topics, services, and actions.
/// Bus only uses it on **topics** (depth → HWM; must be `.best_effort()`).
/// Service / action bus names have no QoS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicQos {
    depth: i32,
    reliability: TopicReliability,
}

impl TopicQos {
    pub fn keep_last(depth: i32) -> TopicQosKeepLast {
        TopicQosKeepLast { depth }
    }

    pub fn depth(self) -> i32 {
        self.depth
    }

    pub fn reliability(self) -> TopicReliability {
        self.reliability
    }

    pub fn is_reliable(self) -> bool {
        matches!(self.reliability, TopicReliability::Reliable)
    }

    pub fn is_best_effort(self) -> bool {
        matches!(self.reliability, TopicReliability::BestEffort)
    }
}

pub(crate) fn ros_topic_options(topic: &str, qos: TopicQos) -> rclrs::PrimitiveOptions<'_> {
    let mut opts = topic
        .into_primitive_options()
        .keep_last(qos.depth().max(0) as u32);
    if qos.is_best_effort() {
        opts = opts.best_effort();
    } else {
        opts = opts.reliable();
    }
    opts
}

pub(crate) fn ros_service_qos_profile(qos: TopicQos) -> rclrs::QoSProfile {
    let mut p = rclrs::QoSProfile::services_default().keep_last(qos.depth().max(0) as u32);
    if qos.is_best_effort() {
        p = p.best_effort();
    } else {
        p = p.reliable();
    }
    p
}

pub(crate) fn ros_action_feedback_qos_profile(qos: TopicQos) -> rclrs::QoSProfile {
    let mut p = rclrs::QoSProfile::topics_default().keep_last(qos.depth().max(0) as u32);
    if qos.is_best_effort() {
        p = p.best_effort();
    } else {
        p = p.reliable();
    }
    p
}

/// Context passed to [`TopicMapper::attach_bus_to_ros`].
pub struct TopicWireContext<'a> {
    pub ros_node: &'a rclrs::Node,
    pub bus_node: &'a mut Node,
    pub ros_topic: &'a str,
    pub bus_topic: &'a str,
    pub ros_qos: TopicQos,
    pub bus_qos: TopicQos,
    pub ros_entities: &'a mut Vec<Box<dyn Any + Send + Sync>>,
}

/// Typed topic codec: ROS IDL object ↔ bus protobuf object.
///
/// The library wires subscriptions and publishers. Implement this for builtins
/// and custom topic types.
pub trait TypedTopicMapper: Clone + Send + Sync + 'static {
    type Ros: RosMessage + Send + Sync + Default + 'static;
    type Bus: ProstMessage + Default + Send + 'static;

    fn type_name(&self) -> &str;
    fn ros_to_bus(&self, msg: Self::Ros) -> BusResult<Self::Bus>;
    fn bus_to_ros(&self, msg: Self::Bus) -> BusResult<Self::Ros>;
}

/// Object-safe topic plugin. Prefer [`TypedTopicMapper`]; the blanket impl
/// wires typed ROS↔bus endpoints.
pub trait TopicMapper: Send + Sync {
    /// Full ROS type name, e.g. `sensor_msgs/msg/Image`.
    fn type_name(&self) -> &str;

    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicQos,
    ) -> Result<Box<dyn Any + Send + Sync>>;

    fn attach_bus_to_ros(&self, ctx: TopicWireContext<'_>) -> Result<()>;
}

impl<T> TopicMapper for T
where
    T: TypedTopicMapper,
{
    fn type_name(&self) -> &str {
        TypedTopicMapper::type_name(self)
    }

    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicQos,
    ) -> Result<Box<dyn Any + Send + Sync>> {
        create_typed_ros2_to_bus_sub(self, ros_node, bus_pub, ros_topic, qos)
    }

    fn attach_bus_to_ros(&self, ctx: TopicWireContext<'_>) -> Result<()> {
        attach_typed_bus_to_ros(self, ctx)
    }
}

/// Context passed to [`ServiceMapper::attach`].
pub struct ServiceWireContext<'a> {
    pub ros_node: &'a rclrs::Node,
    pub bus_node: &'a mut Node,
    pub ros_service: &'a str,
    pub bus_service: &'a str,
    pub direction: Direction,
    pub timeout: Duration,
    pub ros_qos: TopicQos,
    pub ros_entities: &'a mut Vec<Box<dyn Any + Send + Sync>>,
}

/// Custom service codec: convert methods only; library calls [`wire_typed_service`].
pub trait TypedServiceMapper: Clone + Send + Sync + 'static {
    type Ros: ServiceIdl;

    fn type_name(&self) -> &str;

    fn ros_req_to_bus(&self, req: &<Self::Ros as ServiceIdl>::Request) -> BusResult<Vec<u8>>;
    fn bus_req_to_ros(&self, payload: &[u8]) -> BusResult<<Self::Ros as ServiceIdl>::Request>;
    fn ros_resp_to_bus(&self, resp: &<Self::Ros as ServiceIdl>::Response) -> BusResult<Vec<u8>>;
    fn bus_resp_to_ros(&self, payload: &[u8]) -> BusResult<<Self::Ros as ServiceIdl>::Response>;

    /// ROS response used when bridging fails (timeout / decode / bus error).
    fn error_response(&self, message: &str) -> <Self::Ros as ServiceIdl>::Response
    where
        <Self::Ros as ServiceIdl>::Response: Default,
    {
        let _ = message;
        Default::default()
    }
}

/// Service type codec. Prefer [`TypedServiceMapper`]; override `attach` only when needed.
pub trait ServiceMapper: Send + Sync {
    fn type_name(&self) -> &str;

    /// Attach ROS↔bus forwarding. Default dispatches known builtins by `type_name`.
    fn attach(&self, ctx: ServiceWireContext<'_>) -> BusResult<()> {
        typed_rpc::attach_builtin_service(self.type_name(), ctx)
    }
}

impl<T> ServiceMapper for T
where
    T: TypedServiceMapper,
    <T::Ros as ServiceIdl>::Request: Send + Sync + 'static,
    <T::Ros as ServiceIdl>::Response: Send + Sync + Default + 'static,
{
    fn type_name(&self) -> &str {
        TypedServiceMapper::type_name(self)
    }

    fn attach(&self, ctx: ServiceWireContext<'_>) -> BusResult<()> {
        wire_typed_service(self, ctx)
    }
}

/// Context passed to [`ActionMapper::attach`].
pub struct ActionWireContext<'a> {
    pub ros_node: &'a rclrs::Node,
    pub bus_node: &'a mut Node,
    pub ros_action: &'a str,
    pub bus_action: &'a str,
    pub direction: Direction,
    pub timeout: Duration,
    pub ros_qos: TopicQos,
    pub ros_entities: &'a mut Vec<Box<dyn Any + Send + Sync>>,
}

/// Custom action codec: convert methods only; library calls [`wire_typed_action`].
pub trait TypedActionMapper: Clone + Send + Sync + 'static {
    type Ros: ActionIdl;

    fn type_name(&self) -> &str;

    fn ros_goal_to_bus(&self, goal: &<Self::Ros as ActionIdl>::Goal) -> BusResult<Vec<u8>>;
    fn bus_goal_to_ros(&self, payload: &[u8]) -> BusResult<<Self::Ros as ActionIdl>::Goal>;
    fn ros_feedback_to_bus(
        &self,
        feedback: &<Self::Ros as ActionIdl>::Feedback,
    ) -> BusResult<Vec<u8>>;
    fn bus_feedback_to_ros(&self, payload: &[u8]) -> BusResult<<Self::Ros as ActionIdl>::Feedback>;
    fn ros_result_to_bus(&self, result: &<Self::Ros as ActionIdl>::Result) -> BusResult<Vec<u8>>;
    fn bus_result_to_ros(&self, payload: &[u8]) -> BusResult<<Self::Ros as ActionIdl>::Result>;
}

/// Action type codec. Prefer [`TypedActionMapper`]; override `attach` only when needed.
pub trait ActionMapper: Send + Sync {
    fn type_name(&self) -> &str;

    fn attach(&self, ctx: ActionWireContext<'_>) -> BusResult<()> {
        typed_rpc::attach_builtin_action(self.type_name(), ctx)
    }
}

impl<T> ActionMapper for T
where
    T: TypedActionMapper,
    <T::Ros as ActionIdl>::Goal: Clone + Send + Sync + 'static,
    <T::Ros as ActionIdl>::Feedback: Clone + Send + Sync + 'static,
    <T::Ros as ActionIdl>::Result: Default + Clone + Send + Sync + 'static,
{
    fn type_name(&self) -> &str {
        TypedActionMapper::type_name(self)
    }

    fn attach(&self, ctx: ActionWireContext<'_>) -> BusResult<()> {
        wire_typed_action(self, ctx)
    }
}

struct RefTopicMapper(&'static dyn TopicMapper);

impl TopicMapper for RefTopicMapper {
    fn type_name(&self) -> &'static str {
        self.0.type_name()
    }

    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicQos,
    ) -> Result<Box<dyn Any + Send + Sync>> {
        self.0
            .create_ros2_to_bus_subscription(ros_node, bus_pub, ros_topic, qos)
    }

    fn attach_bus_to_ros(&self, ctx: TopicWireContext<'_>) -> Result<()> {
        self.0.attach_bus_to_ros(ctx)
    }
}

static BUILTIN_MAPPERS: LazyLock<HashMap<&'static str, &'static dyn TopicMapper>> =
    LazyLock::new(|| {
        let mut map = HashMap::with_capacity(BUILTIN_MAPPER_LIST.len());
        for m in BUILTIN_MAPPER_LIST {
            map.insert(m.type_name(), *m);
        }
        map
    });

/// Look up a registered topic mapper by ROS type string (registry introspection / tests).
///
/// Not a route-mounting API — use `.mapper(ConcreteMapper)` on the builder.
pub fn lookup_topic_mapper(type_name: &str) -> Result<&'static dyn TopicMapper> {
    BUILTIN_MAPPERS.get(type_name).copied().ok_or_else(|| {
        BusError::Protocol(format!(
            "unsupported ros2 bridge topic type {type_name:?}; \
             registered types mirror proto/*/msg/v1 ({} total), see registered_topic_types(); \
             for custom types use .mapper(...) on the route",
            BUILTIN_MAPPERS.len()
        ))
    })
}

/// Builtin topic mapper as [`Arc`] (registry / tests).
pub fn lookup_topic_mapper_arc(type_name: &str) -> Result<Arc<dyn TopicMapper>> {
    Ok(Arc::new(RefTopicMapper(lookup_topic_mapper(type_name)?)))
}

/// Sorted list of registered topic type names (for docs / errors / tests).
pub fn registered_topic_types() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = BUILTIN_MAPPERS.keys().copied().collect();
    v.sort_unstable();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_qos_requires_reliability() {
        let reliable = TopicQos::keep_last(10).reliable();
        assert_eq!(reliable.depth(), 10);
        assert!(reliable.is_reliable());
        assert!(!reliable.is_best_effort());
        let be = TopicQos::keep_last(5).best_effort();
        assert_eq!(be.depth(), 5);
        assert!(be.is_best_effort());
    }

    #[test]
    fn registry_covers_proto_message_types() {
        let types = registered_topic_types();
        assert!(
            types.len() >= 150,
            "expected the registry to mirror proto/*/msg/v1, got {}",
            types.len()
        );
        for expected in [
            "builtin_interfaces/msg/Time",
            "foxglove_msgs/msg/CompressedVideo",
            "geometry_msgs/msg/PoseStamped",
            "std_msgs/msg/String",
            "sensor_msgs/msg/Image",
        ] {
            assert!(types.contains(&expected), "{expected} not registered");
        }
    }

    #[test]
    fn registry_keys_match_mapper_type_names() {
        for t in registered_topic_types() {
            assert_eq!(lookup_topic_mapper(t).unwrap().type_name(), t);
        }
    }

    #[test]
    fn lookup_unknown_reports_unsupported() {
        let Err(e) = lookup_topic_mapper("my_pkg/msg/Foo") else {
            panic!("expected lookup failure");
        };
        assert!(e.to_string().contains("unsupported"));
    }
}
