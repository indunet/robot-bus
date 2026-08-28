//! Typed mapper for `foxglove_msgs/msg/Vector3`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn vector3_to_bus(msg: ros_env::foxglove_msgs::msg::Vector3) -> crate::foxglove_msgs::msg::v1::Vector3 {
    crate::foxglove_msgs::msg::v1::Vector3 {
        x: msg.x,
        y: msg.y,
        z: msg.z,
    }
}

pub(crate) fn vector3_to_ros(bus: crate::foxglove_msgs::msg::v1::Vector3) -> ros_env::foxglove_msgs::msg::Vector3 {
    ros_env::foxglove_msgs::msg::Vector3 {
        x: bus.x,
        y: bus.y,
        z: bus.z,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsVector3Mapper;

impl TypedTopicMapper for FoxgloveMsgsVector3Mapper {
    type Ros = ros_env::foxglove_msgs::msg::Vector3;
    type Bus = crate::foxglove_msgs::msg::v1::Vector3;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Vector3"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(vector3_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(vector3_to_ros(msg))
    }
}
