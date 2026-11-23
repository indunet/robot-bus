//! Typed mapper for `geometry_msgs/msg/Quaternion`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn quaternion_to_bus(
    msg: ros_env::geometry_msgs::msg::Quaternion,
) -> robot_bus::geometry_msgs::msg::v1::Quaternion {
    robot_bus::geometry_msgs::msg::v1::Quaternion {
        x: msg.x,
        y: msg.y,
        z: msg.z,
        w: msg.w,
    }
}

pub(crate) fn quaternion_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Quaternion,
) -> ros_env::geometry_msgs::msg::Quaternion {
    ros_env::geometry_msgs::msg::Quaternion {
        x: bus.x,
        y: bus.y,
        z: bus.z,
        w: bus.w,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsQuaternionMapper;

impl TypedTopicMapper for GeometryMsgsQuaternionMapper {
    type Ros = ros_env::geometry_msgs::msg::Quaternion;
    type Bus = robot_bus::geometry_msgs::msg::v1::Quaternion;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(quaternion_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(quaternion_to_ros(msg))
    }
}
