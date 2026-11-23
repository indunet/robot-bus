//! Typed mapper for `std_msgs/msg/UInt32`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn uint32_to_bus(
    msg: ros_env::std_msgs::msg::UInt32,
) -> robot_bus::std_msgs::msg::v1::UInt32 {
    robot_bus::std_msgs::msg::v1::UInt32 {
        data: msg.data.into(),
    }
}

pub(crate) fn uint32_to_ros(
    bus: robot_bus::std_msgs::msg::v1::UInt32,
) -> ros_env::std_msgs::msg::UInt32 {
    ros_env::std_msgs::msg::UInt32 {
        data: bus.data as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt32Mapper;

impl TypedTopicMapper for StdMsgsUInt32Mapper {
    type Ros = ros_env::std_msgs::msg::UInt32;
    type Bus = robot_bus::std_msgs::msg::v1::UInt32;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(uint32_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(uint32_to_ros(msg))
    }
}
