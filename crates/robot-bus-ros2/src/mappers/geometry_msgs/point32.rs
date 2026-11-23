//! Typed mapper for `geometry_msgs/msg/Point32`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn point32_to_bus(
    msg: ros_env::geometry_msgs::msg::Point32,
) -> robot_bus::geometry_msgs::msg::v1::Point32 {
    robot_bus::geometry_msgs::msg::v1::Point32 {
        x: msg.x,
        y: msg.y,
        z: msg.z,
    }
}

pub(crate) fn point32_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Point32,
) -> ros_env::geometry_msgs::msg::Point32 {
    ros_env::geometry_msgs::msg::Point32 {
        x: bus.x,
        y: bus.y,
        z: bus.z,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoint32Mapper;

impl TypedTopicMapper for GeometryMsgsPoint32Mapper {
    type Ros = ros_env::geometry_msgs::msg::Point32;
    type Bus = robot_bus::geometry_msgs::msg::v1::Point32;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(point32_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(point32_to_ros(msg))
    }
}
