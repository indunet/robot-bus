//! Typed mapper for `std_msgs/msg/UInt16`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn uint16_to_bus(
    msg: ros_env::std_msgs::msg::UInt16,
) -> robot_bus::std_msgs::msg::v1::UInt16 {
    robot_bus::std_msgs::msg::v1::UInt16 {
        data: u32::from(msg.data),
    }
}

pub(crate) fn uint16_to_ros(
    bus: robot_bus::std_msgs::msg::v1::UInt16,
) -> ros_env::std_msgs::msg::UInt16 {
    ros_env::std_msgs::msg::UInt16 {
        data: bus.data as u16,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt16Mapper;

impl TypedTopicMapper for StdMsgsUInt16Mapper {
    type Ros = ros_env::std_msgs::msg::UInt16;
    type Bus = robot_bus::std_msgs::msg::v1::UInt16;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(uint16_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(uint16_to_ros(msg))
    }
}
