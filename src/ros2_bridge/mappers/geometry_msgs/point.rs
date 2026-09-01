//! Typed mapper for `geometry_msgs/msg/Point`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_to_bus(msg: ros_env::geometry_msgs::msg::Point) -> crate::geometry_msgs::msg::v1::Point {
    crate::geometry_msgs::msg::v1::Point {
        x: msg.x,
        y: msg.y,
        z: msg.z,
    }
}

pub(crate) fn point_to_ros(bus: crate::geometry_msgs::msg::v1::Point) -> ros_env::geometry_msgs::msg::Point {
    ros_env::geometry_msgs::msg::Point {
        x: bus.x,
        y: bus.y,
        z: bus.z,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPointMapper;

impl TypedTopicMapper for GeometryMsgsPointMapper {
    type Ros = ros_env::geometry_msgs::msg::Point;
    type Bus = crate::geometry_msgs::msg::v1::Point;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_to_ros(msg))
    }
}
