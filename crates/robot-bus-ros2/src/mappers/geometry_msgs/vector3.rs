//! Typed mapper for `geometry_msgs/msg/Vector3`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn vector3_to_bus(
    msg: ros_env::geometry_msgs::msg::Vector3,
) -> robot_bus::geometry_msgs::msg::v1::Vector3 {
    robot_bus::geometry_msgs::msg::v1::Vector3 {
        x: msg.x,
        y: msg.y,
        z: msg.z,
    }
}

pub(crate) fn vector3_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Vector3,
) -> ros_env::geometry_msgs::msg::Vector3 {
    ros_env::geometry_msgs::msg::Vector3 {
        x: bus.x,
        y: bus.y,
        z: bus.z,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsVector3Mapper;

impl TypedTopicMapper for GeometryMsgsVector3Mapper {
    type Ros = ros_env::geometry_msgs::msg::Vector3;
    type Bus = robot_bus::geometry_msgs::msg::v1::Vector3;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(vector3_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(vector3_to_ros(msg))
    }
}
