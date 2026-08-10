//! Topic / service / action mapper traits and topic builtin registry.
//!
//! Topics use [`TopicMapper`] (DynamicMessage ↔ protobuf). Services / actions use
//! [`ServiceMapper`] / [`ActionMapper`] as **type codecs** (identify a ROS type;
//! field converters live beside builtins). The library attaches typed ROS
//! client/server entities for known builtins via [`super::typed_rpc`]. Arbitrary
//! custom codecs need dynamic service/action support (Track B) or an `attach`
//! override with a Rust typed backend.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use rclrs::{DynamicMessage, MessageTypeName};

use crate::errors::{BusError, Result as BusResult};
use crate::runtime::Node;

use super::mappers::BUILTIN_MAPPER_LIST;
use super::typed_rpc;

type Result<T> = std::result::Result<T, BusError>;

/// Topic / service / action bridge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ros2ToBus,
    BusToRos2,
}

/// Bidirectional mapper between ROS [`DynamicMessage`] and bus protobuf bytes.
pub trait TopicMapper: Send + Sync {
    /// Full ROS type name, e.g. `sensor_msgs/msg/Image`.
    ///
    /// Builtin mappers return a `'static` string; FFI / owned wrappers may return
    /// a borrow of an owned `String`.
    fn type_name(&self) -> &str;

    fn ros_type(&self) -> MessageTypeName {
        MessageTypeName::try_from(self.type_name())
            .expect("TopicMapper::type_name must be package/msg/Type")
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>>;
    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage>;
}

/// Context passed to [`ServiceMapper::attach`] (library / advanced typed backends).
pub struct ServiceWireContext<'a> {
    pub ros_node: &'a rclrs::Node,
    pub bus_node: &'a mut Node,
    pub ros_service: &'a str,
    pub bus_service: &'a str,
    pub direction: Direction,
    pub timeout: Duration,
    pub ros_entities: &'a mut Vec<Box<dyn Any + Send + Sync>>,
}

/// Service type codec: identifies a ROS service type for library-owned wiring.
///
/// Builtin ZSTs (e.g. [`crate::ros2_bridge::TriggerServiceMapper`]) only implement
/// [`type_name`](ServiceMapper::type_name); [`attach`](ServiceMapper::attach)
/// defaults to the typed builtin backend. Override `attach` for a custom Rust
/// typed backend. Arbitrary codecs without a typed backend need Track B.
pub trait ServiceMapper: Send + Sync {
    fn type_name(&self) -> &str;

    /// Attach ROS↔bus forwarding. Default: builtin typed backends by `type_name`.
    fn attach(&self, ctx: ServiceWireContext<'_>) -> BusResult<()> {
        typed_rpc::attach_builtin_service(self.type_name(), ctx)
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

/// Action type codec: identifies a ROS action type for library-owned wiring.
pub trait ActionMapper: Send + Sync {
    fn type_name(&self) -> &str;

    /// Attach ROS↔bus forwarding. Default: builtin typed backends by `type_name`.
    fn attach(&self, ctx: ActionWireContext<'_>) -> BusResult<()> {
        typed_rpc::attach_builtin_action(self.type_name(), ctx)
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

/// Look up a registered topic mapper by ROS type string.
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

/// Builtin topic mapper as [`Arc`] (for per-route storage alongside custom mappers).
pub fn lookup_topic_mapper_arc(type_name: &str) -> Result<Arc<dyn TopicMapper>> {
    Ok(Arc::new(RefTopicMapper(lookup_topic_mapper(type_name)?)))
}

/// Sorted list of registered topic type names (for docs / errors).
pub fn registered_topic_types() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = BUILTIN_MAPPERS.keys().copied().collect();
    v.sort_unstable();
    v
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::foxglove_msgs::msg::v1::CompressedVideo as BusCompressedVideo;
    use crate::sensor_msgs::msg::v1::Image as BusImage;
    use prost::Message as ProstMessage;
    use prost_types::Timestamp;

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
            "geometry_msgs/msg/Twist",
            "nav_msgs/msg/OccupancyGrid",
            "nav_msgs/msg/Odometry",
            "nav_msgs/msg/Path",
            "sensor_msgs/msg/CompressedImage",
            "sensor_msgs/msg/Image",
            "sensor_msgs/msg/Imu",
            "sensor_msgs/msg/JointState",
            "sensor_msgs/msg/LaserScan",
            "sensor_msgs/msg/PointCloud2",
            "std_msgs/msg/String",
            "tf2_msgs/msg/TFMessage",
            "visualization_msgs/msg/MarkerArray",
        ] {
            assert!(types.contains(&expected), "{expected} not registered");
        }
        assert!(
            !types.iter().any(|t| t.starts_with("robot_bus_interface/")),
            "bus-internal types must stay out of the ROS registry"
        );
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
        let err = e.to_string();
        assert!(err.contains("unsupported"));
        assert!(err.contains("my_pkg/msg/Foo"));
    }

    #[test]
    fn image_roundtrip_when_typesupport_available() {
        let mapper = lookup_topic_mapper("sensor_msgs/msg/Image").unwrap();
        let bus = BusImage {
            header: Some(crate::std_msgs::msg::v1::Header {
                frame_id: "cam".into(),
                stamp: Some(crate::builtin_interfaces::msg::v1::Time { sec: 1, nanosec: 2 }),
            }),
            height: 2,
            width: 2,
            encoding: "rgb8".into(),
            is_bigendian: false,
            step: 6,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        };
        let Ok(dyn_msg) = mapper.bus_to_ros(&bus.encode_to_vec()) else {
            // No ROS type support in this environment (e.g. ros2-shim).
            return;
        };
        let back = BusImage::decode(mapper.ros_to_bus(&dyn_msg).unwrap().as_slice()).unwrap();
        assert_eq!(back.height, 2);
        assert_eq!(back.width, 2);
        assert_eq!(back.encoding, "rgb8");
        assert!(!back.is_bigendian);
        assert_eq!(back.step, 6);
        assert_eq!(back.data, bus.data);
        assert_eq!(back.header.as_ref().unwrap().frame_id, "cam");
    }

    #[test]
    fn compressed_video_roundtrip_when_typesupport_available() {
        let mapper = lookup_topic_mapper("foxglove_msgs/msg/CompressedVideo").unwrap();
        let bus = BusCompressedVideo {
            timestamp: Some(Timestamp {
                seconds: 10,
                nanos: 20,
            }),
            frame_id: "cam".into(),
            data: vec![0x00, 0x00, 0x00, 0x01, 0x67],
            format: "h264".into(),
        };
        let Ok(dyn_msg) = mapper.bus_to_ros(&bus.encode_to_vec()) else {
            return;
        };
        let back =
            BusCompressedVideo::decode(mapper.ros_to_bus(&dyn_msg).unwrap().as_slice()).unwrap();
        assert_eq!(back.frame_id, "cam");
        assert_eq!(back.format, "h264");
        assert_eq!(back.data, bus.data);
        let ts = back.timestamp.unwrap();
        assert_eq!(ts.seconds, 10);
        assert_eq!(ts.nanos, 20);
    }

    #[test]
    fn point_cloud2_roundtrip_when_typesupport_available() {
        let mapper = lookup_topic_mapper("sensor_msgs/msg/PointCloud2").unwrap();
        let bus = crate::sensor_msgs::msg::v1::PointCloud2 {
            header: Some(crate::std_msgs::msg::v1::Header {
                frame_id: "lidar".into(),
                stamp: None,
            }),
            height: 1,
            width: 2,
            fields: vec![crate::sensor_msgs::msg::v1::PointField {
                name: "x".into(),
                offset: 0,
                datatype: 7,
                count: 1,
            }],
            is_bigendian: false,
            point_step: 4,
            row_step: 8,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            is_dense: true,
        };
        let Ok(dyn_msg) = mapper.bus_to_ros(&bus.encode_to_vec()) else {
            return;
        };
        let back = crate::sensor_msgs::msg::v1::PointCloud2::decode(
            mapper.ros_to_bus(&dyn_msg).unwrap().as_slice(),
        )
        .unwrap();
        assert_eq!(back.width, 2);
        assert_eq!(back.data, bus.data);
        assert_eq!(back.fields.len(), 1);
        assert_eq!(back.fields[0].name, "x");
        assert!(back.is_dense);
    }

    #[test]
    fn occupancy_grid_roundtrip_when_typesupport_available() {
        let mapper = lookup_topic_mapper("nav_msgs/msg/OccupancyGrid").unwrap();
        let bus = crate::nav_msgs::msg::v1::OccupancyGrid {
            header: None,
            info: Some(crate::nav_msgs::msg::v1::MapMetaData {
                map_load_time: None,
                resolution: 0.05,
                width: 2,
                height: 2,
                origin: None,
            }),
            // int8[] on the ROS side: negative "unknown" cells must survive widening.
            data: vec![-1, 0, 50, 100],
        };
        let Ok(dyn_msg) = mapper.bus_to_ros(&bus.encode_to_vec()) else {
            return;
        };
        let back = crate::nav_msgs::msg::v1::OccupancyGrid::decode(
            mapper.ros_to_bus(&dyn_msg).unwrap().as_slice(),
        )
        .unwrap();
        assert_eq!(back.data, vec![-1, 0, 50, 100]);
        assert_eq!(back.info.unwrap().width, 2);
    }

    #[test]
    fn path_roundtrip_when_typesupport_available() {
        let mapper = lookup_topic_mapper("nav_msgs/msg/Path").unwrap();
        let pose = crate::geometry_msgs::msg::v1::PoseStamped {
            header: Some(crate::std_msgs::msg::v1::Header {
                frame_id: "map".into(),
                stamp: None,
            }),
            pose: Some(crate::geometry_msgs::msg::v1::Pose {
                position: Some(crate::geometry_msgs::msg::v1::Point {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                }),
                orientation: Some(crate::geometry_msgs::msg::v1::Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }),
            }),
        };
        let bus = crate::nav_msgs::msg::v1::Path {
            header: None,
            poses: vec![pose.clone(), pose],
        };
        let Ok(dyn_msg) = mapper.bus_to_ros(&bus.encode_to_vec()) else {
            return;
        };
        let back =
            crate::nav_msgs::msg::v1::Path::decode(mapper.ros_to_bus(&dyn_msg).unwrap().as_slice())
                .unwrap();
        assert_eq!(back.poses.len(), 2);
        let position = back.poses[0]
            .pose
            .as_ref()
            .unwrap()
            .position
            .as_ref()
            .unwrap();
        assert_eq!(position.x, 1.0);
        assert_eq!(position.z, 3.0);
    }
}
