//! Typed mapper for `std_msgs/msg/UInt64`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uint64_to_bus(
    msg: ros_env::std_msgs::msg::UInt64,
) -> crate::std_msgs::msg::v1::UInt64 {
    crate::std_msgs::msg::v1::UInt64 { data: msg.data }
}

pub(crate) fn uint64_to_ros(
    bus: crate::std_msgs::msg::v1::UInt64,
) -> ros_env::std_msgs::msg::UInt64 {
    ros_env::std_msgs::msg::UInt64 { data: bus.data }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt64Mapper;

impl TypedTopicMapper for StdMsgsUInt64Mapper {
    type Ros = ros_env::std_msgs::msg::UInt64;
    type Bus = crate::std_msgs::msg::v1::UInt64;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uint64_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uint64_to_ros(msg))
    }
}
