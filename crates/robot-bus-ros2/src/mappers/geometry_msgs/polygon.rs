//! Typed mapper for `geometry_msgs/msg/Polygon`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn polygon_to_bus(
    msg: ros_env::geometry_msgs::msg::Polygon,
) -> robot_bus::geometry_msgs::msg::v1::Polygon {
    robot_bus::geometry_msgs::msg::v1::Polygon {
        points: msg
            .points
            .into_iter()
            .map(crate::mappers::geometry_msgs::point32::point32_to_bus)
            .collect(),
    }
}

pub(crate) fn polygon_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Polygon,
) -> ros_env::geometry_msgs::msg::Polygon {
    ros_env::geometry_msgs::msg::Polygon {
        points: bus
            .points
            .into_iter()
            .map(crate::mappers::geometry_msgs::point32::point32_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPolygonMapper;

impl TypedTopicMapper for GeometryMsgsPolygonMapper {
    type Ros = ros_env::geometry_msgs::msg::Polygon;
    type Bus = robot_bus::geometry_msgs::msg::v1::Polygon;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(polygon_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(polygon_to_ros(msg))
    }
}
