//! Typed mapper for `std_msgs/msg/Float32`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn float32_to_bus(
    msg: ros_env::std_msgs::msg::Float32,
) -> crate::std_msgs::msg::v1::Float32 {
    crate::std_msgs::msg::v1::Float32 { data: msg.data }
}

pub(crate) fn float32_to_ros(
    bus: crate::std_msgs::msg::v1::Float32,
) -> ros_env::std_msgs::msg::Float32 {
    ros_env::std_msgs::msg::Float32 { data: bus.data }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsFloat32Mapper;

impl TypedTopicMapper for StdMsgsFloat32Mapper {
    type Ros = ros_env::std_msgs::msg::Float32;
    type Bus = crate::std_msgs::msg::v1::Float32;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(float32_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(float32_to_ros(msg))
    }
}
