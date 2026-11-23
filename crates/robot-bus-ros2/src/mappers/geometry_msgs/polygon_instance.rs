//! Typed mapper for `geometry_msgs/msg/PolygonInstance`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn polygon_instance_to_bus(
    msg: ros_env::geometry_msgs::msg::PolygonInstance,
) -> robot_bus::geometry_msgs::msg::v1::PolygonInstance {
    robot_bus::geometry_msgs::msg::v1::PolygonInstance {
        polygon: Some(
            crate::mappers::geometry_msgs::polygon::polygon_to_bus(msg.polygon),
        ),
        id: msg.id,
    }
}

pub(crate) fn polygon_instance_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::PolygonInstance,
) -> ros_env::geometry_msgs::msg::PolygonInstance {
    ros_env::geometry_msgs::msg::PolygonInstance {
        polygon: crate::mappers::geometry_msgs::polygon::polygon_to_ros(
            bus.polygon.unwrap_or_default(),
        ),
        id: bus.id,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPolygonInstanceMapper;

impl TypedTopicMapper for GeometryMsgsPolygonInstanceMapper {
    type Ros = ros_env::geometry_msgs::msg::PolygonInstance;
    type Bus = robot_bus::geometry_msgs::msg::v1::PolygonInstance;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(polygon_instance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(polygon_instance_to_ros(msg))
    }
}
