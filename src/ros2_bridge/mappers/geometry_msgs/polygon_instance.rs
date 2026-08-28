//! Typed mapper for `geometry_msgs/msg/PolygonInstance`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn polygon_instance_to_bus(msg: ros_env::geometry_msgs::msg::PolygonInstance) -> crate::geometry_msgs::msg::v1::PolygonInstance {
    crate::geometry_msgs::msg::v1::PolygonInstance {
        polygon: Some(crate::ros2_bridge::mappers::geometry_msgs::polygon::polygon_to_bus(msg.polygon)),
        id: msg.id,
    }
}

pub(crate) fn polygon_instance_to_ros(bus: crate::geometry_msgs::msg::v1::PolygonInstance) -> ros_env::geometry_msgs::msg::PolygonInstance {
    ros_env::geometry_msgs::msg::PolygonInstance {
        polygon: crate::ros2_bridge::mappers::geometry_msgs::polygon::polygon_to_ros(bus.polygon.unwrap_or_default()),
        id: bus.id,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPolygonInstanceMapper;

impl TypedTopicMapper for GeometryMsgsPolygonInstanceMapper {
    type Ros = ros_env::geometry_msgs::msg::PolygonInstance;
    type Bus = crate::geometry_msgs::msg::v1::PolygonInstance;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PolygonInstance"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(polygon_instance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(polygon_instance_to_ros(msg))
    }
}
