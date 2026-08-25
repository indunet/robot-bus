//! Topic / service / action mapper traits and topic builtin registry.
//!
//! Topics use [`TopicMapper`] (DynamicMessage ↔ protobuf). Services / actions:
//! implement [`TypedServiceMapper`] / [`TypedActionMapper`] (convert methods only);
//! the library wires ROS↔bus. Override [`ServiceMapper::attach`] /
//! [`ActionMapper::attach`] only for advanced cases.
//! Route mounting is always via concrete mapper objects (`.mapper(...)`).

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use rclrs::{DynamicMessage, IntoPrimitiveOptions, MessageTypeName};
use rosidl_runtime_rs::{Action as ActionIdl, Service as ServiceIdl};

use crate::errors::{BusError, Result as BusResult};
use crate::runtime::{Node, TopicPublisherRaw};

use super::mappers::BUILTIN_MAPPER_LIST;
use super::typed_rpc;
use super::typed_wire::{wire_typed_action, wire_typed_service};

type Result<T> = std::result::Result<T, BusError>;

/// Topic / service / action bridge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ros2ToBus,
    BusToRos2,
}

/// Opt-in ROS + bus topic QoS for a bridge route. Default leaves both stacks unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TopicRouteQos {
    /// ROS KeepLast depth and bus HWM when set.
    pub depth: Option<i32>,
    /// ROS reliability best-effort when true.
    pub best_effort: bool,
    /// Use ROS `SensorDataQoS` (best-effort KeepLast 5) plus bus depth 5.
    pub sensor_data: bool,
}

impl TopicRouteQos {
    /// rclcpp `SensorDataQoS` history: best-effort KeepLast(5).
    pub const SENSOR_DATA: Self = Self {
        depth: Some(5),
        best_effort: true,
        sensor_data: true,
    };
}

pub(crate) fn ros_topic_options(topic: &str, qos: TopicRouteQos) -> rclrs::PrimitiveOptions<'_> {
    if qos.sensor_data {
        return topic.sensor_data_qos();
    }
    let mut opts = topic.into_primitive_options();
    if let Some(depth) = qos.depth {
        opts = opts.keep_last(depth.max(0) as u32);
    }
    if qos.best_effort {
        opts = opts.best_effort();
    }
    opts
}

/// Bidirectional mapper between ROS [`DynamicMessage`] and bus protobuf bytes.
pub trait TopicMapper: Send + Sync {
    /// Full ROS type name, e.g. `sensor_msgs/msg/Image` (for DynamicMessage create).
    fn type_name(&self) -> &str;

    fn ros_type(&self) -> MessageTypeName {
        MessageTypeName::try_from(self.type_name())
            .expect("TopicMapper::type_name must be package/msg/Type")
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>>;
    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage>;

    /// Optional typed ROS→bus subscription. `Ok(None)` uses the DynamicMessage path.
    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: TopicPublisherRaw,
        ros_topic: &str,
        qos: TopicRouteQos,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>> {
        let _ = (ros_node, bus_pub, ros_topic, qos);
        Ok(None)
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
    fn bus_feedback_to_ros(
        &self,
        payload: &[u8],
    ) -> BusResult<<Self::Ros as ActionIdl>::Feedback>;
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
    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        self.0.ros_to_bus(msg)
    }
    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        self.0.bus_to_ros(payload)
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
