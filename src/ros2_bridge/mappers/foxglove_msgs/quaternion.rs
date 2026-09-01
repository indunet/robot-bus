//! Typed mapper for `foxglove_msgs/msg/Quaternion`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn quaternion_to_bus(msg: ros_env::foxglove_msgs::msg::Quaternion) -> crate::foxglove_msgs::msg::v1::Quaternion {
    crate::foxglove_msgs::msg::v1::Quaternion {
        x: msg.x,
        y: msg.y,
        z: msg.z,
        w: msg.w,
    }
}

pub(crate) fn quaternion_to_ros(bus: crate::foxglove_msgs::msg::v1::Quaternion) -> ros_env::foxglove_msgs::msg::Quaternion {
    ros_env::foxglove_msgs::msg::Quaternion {
        x: bus.x,
        y: bus.y,
        z: bus.z,
        w: bus.w,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsQuaternionMapper;

impl TypedTopicMapper for FoxgloveMsgsQuaternionMapper {
    type Ros = ros_env::foxglove_msgs::msg::Quaternion;
    type Bus = crate::foxglove_msgs::msg::v1::Quaternion;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(quaternion_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(quaternion_to_ros(msg))
    }
}
