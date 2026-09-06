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
use crate::ros2_bridge::drop_stats::DropStats;
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

/// Durability for [`TopicQos`]. Defaults to [`Volatile`](TopicDurability::Volatile);
/// call [`.transient_local()`](TopicQos::transient_local) for latched ROS topics
/// (`/tf_static`, maps, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicDurability {
    Volatile,
    TransientLocal,
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
            durability: TopicDurability::Volatile,
        }
    }

    pub fn best_effort(self) -> TopicQos {
        TopicQos {
            depth: self.depth,
            reliability: TopicReliability::BestEffort,
            durability: TopicDurability::Volatile,
        }
    }
}

/// Bridge QoS: KeepLast depth plus reliability and optional ROS durability.
///
/// Same type on **ROS** and **bus** endpoints for topics, services, and actions.
/// ROS honors depth + reliability + durability. Bus uses depth as ZMQ HWM
/// (PUB/SUB or DEALER) and must be [`.best_effort()`](TopicQosKeepLast::best_effort)
/// (no DDS reliability); durability is ignored on bus.
///
/// Usual routes use named presets ([`sensor_data`](Self::sensor_data),
/// [`default`](Self::default), [`latched`](Self::latched), [`bus`](Self::bus)).
/// Custom depth still uses [`keep_last`](Self::keep_last).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicQos {
    depth: i32,
    reliability: TopicReliability,
    durability: TopicDurability,
}

impl TopicQos {
    pub fn keep_last(depth: i32) -> TopicQosKeepLast {
        TopicQosKeepLast { depth }
    }

    /// ROS `SensorDataQoS`: KeepLast 5, best effort, volatile.
    pub fn sensor_data() -> Self {
        Self::keep_last(5).best_effort()
    }

    /// ROS topic/service default (`qos_profile_default` / `ServicesQoS`):
    /// KeepLast 10, reliable, volatile.
    ///
    /// Not ROS `SystemDefaultsQoS` (RMW/DDS vendor defaults).
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::keep_last(10).reliable()
    }

    /// Latched ROS topics such as `/tf_static`: KeepLast 1, reliable, transient local.
    pub fn latched() -> Self {
        Self::keep_last(1).reliable().transient_local()
    }

    /// Typical bus endpoint: KeepLast [`crate::HighWaterMark::STREAM`] (8), best effort.
    pub fn bus() -> Self {
        Self::keep_last(crate::HighWaterMark::STREAM.snd).best_effort()
    }

    pub fn depth(self) -> i32 {
        self.depth
    }

    pub fn reliability(self) -> TopicReliability {
        self.reliability
    }

    pub fn durability(self) -> TopicDurability {
        self.durability
    }

    pub fn is_reliable(self) -> bool {
        matches!(self.reliability, TopicReliability::Reliable)
    }

    pub fn is_best_effort(self) -> bool {
        matches!(self.reliability, TopicReliability::BestEffort)
    }

    pub fn is_transient_local(self) -> bool {
        matches!(self.durability, TopicDurability::TransientLocal)
    }

    pub fn is_volatile(self) -> bool {
        matches!(self.durability, TopicDurability::Volatile)
    }

    /// ROS `TRANSIENT_LOCAL` (latch). Needed to receive already-published samples
    /// from latched publishers, and to match subscribers that request it.
    pub fn transient_local(self) -> Self {
        Self {
            durability: TopicDurability::TransientLocal,
            ..self
        }
    }

    /// ROS `VOLATILE` (default). New subscribers only see subsequent samples.
    pub fn volatile(self) -> Self {
        Self {
            durability: TopicDurability::Volatile,
            ..self
        }
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
    if qos.is_transient_local() {
        opts = opts.transient_local();
    } else {
        opts = opts.volatile();
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
    if qos.is_transient_local() {
        p = p.transient_local();
    } else {
        p = p.volatile();
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
    if qos.is_transient_local() {
        p = p.transient_local();
    } else {
        p = p.volatile();
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
    pub drop_stats: Arc<DropStats>,
}

/// Typed topic codec: ROS IDL object ↔ bus protobuf object.
///
/// The library wires subscriptions and publishers. Implement this for builtins
/// and custom topic types.
pub trait TypedTopicMapper: Clone + Send + Sync + 'static {
    type Ros: RosMessage + Send + Sync + Default + 'static;
    type Bus: ProstMessage + Default + Send + 'static;

    fn ros_to_bus(&self, msg: Self::Ros) -> BusResult<Self::Bus>;
    fn bus_to_ros(&self, msg: Self::Bus) -> BusResult<Self::Ros>;
}

/// Object-safe topic plugin. Prefer [`TypedTopicMapper`]; the blanket impl
/// wires typed ROS↔bus endpoints.
pub trait TopicMapper: Send + Sync {
    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicQos,
        drop_stats: Arc<DropStats>,
    ) -> Result<Box<dyn Any + Send + Sync>>;

    fn attach_bus_to_ros(&self, ctx: TopicWireContext<'_>) -> Result<()>;
}

impl<T> TopicMapper for T
where
    T: TypedTopicMapper,
{
    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicQos,
        drop_stats: Arc<DropStats>,
    ) -> Result<Box<dyn Any + Send + Sync>> {
        create_typed_ros2_to_bus_sub(self, ros_node, bus_pub, ros_topic, qos, drop_stats)
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
    pub bus_qos: TopicQos,
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
    pub bus_qos: TopicQos,
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
    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicQos,
        drop_stats: Arc<DropStats>,
    ) -> Result<Box<dyn Any + Send + Sync>> {
        self.0
            .create_ros2_to_bus_subscription(ros_node, bus_pub, ros_topic, qos, drop_stats)
    }

    fn attach_bus_to_ros(&self, ctx: TopicWireContext<'_>) -> Result<()> {
        self.0.attach_bus_to_ros(ctx)
    }
}

static BUILTIN_MAPPERS: LazyLock<HashMap<&'static str, &'static dyn TopicMapper>> =
    LazyLock::new(|| {
        let mut map = HashMap::with_capacity(BUILTIN_MAPPER_LIST.len());
        for (name, m) in BUILTIN_MAPPER_LIST {
            map.insert(*name, *m);
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
             registered types are Humble/Jazzy core bridge builtins ({} total), see registered_topic_types(); \
             for custom / extension types use .mapper(...) on the route",
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
        let latched = TopicQos::keep_last(1).reliable().transient_local();
        assert!(latched.is_transient_local());
        assert!(!latched.is_volatile());
        assert!(!TopicQos::keep_last(10).reliable().is_transient_local());
        assert_eq!(TopicQos::sensor_data(), TopicQos::keep_last(5).best_effort());
        assert_eq!(TopicQos::default(), TopicQos::keep_last(10).reliable());
        assert_eq!(
            TopicQos::latched(),
            TopicQos::keep_last(1).reliable().transient_local()
        );
        assert_eq!(TopicQos::bus(), TopicQos::keep_last(8).best_effort());
        assert_eq!(TopicQos::bus().depth(), crate::HighWaterMark::STREAM.snd);
    }

    #[test]
    fn registry_covers_core_distro_message_types() {
        let types = registered_topic_types();
        assert!(
            (120..=130).contains(&types.len()),
            "expected ~125 Humble/Jazzy core bridge types, got {}",
            types.len()
        );
        for expected in [
            "builtin_interfaces/msg/Time",
            "geometry_msgs/msg/PoseStamped",
            "std_msgs/msg/String",
            "sensor_msgs/msg/Image",
            "nav_msgs/msg/Odometry",
            "visualization_msgs/msg/Marker",
        ] {
            assert!(types.contains(&expected), "{expected} not registered");
        }
        for unexpected in [
            "foxglove_msgs/msg/CompressedVideo",
            "nav2_msgs/msg/Costmap",
            "control_msgs/msg/JointJog",
            "apriltag_msgs/msg/AprilTagDetection",
        ] {
            assert!(
                !types.contains(&unexpected),
                "{unexpected} must not be a default bridge builtin"
            );
        }
    }

    #[test]
    fn registry_lookup_succeeds_for_each_key() {
        for t in registered_topic_types() {
            assert!(lookup_topic_mapper(t).is_ok(), "{t}");
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
