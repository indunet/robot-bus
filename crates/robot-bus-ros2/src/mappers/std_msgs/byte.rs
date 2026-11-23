//! Typed mapper for `std_msgs/msg/Byte`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn byte_to_bus(msg: ros_env::std_msgs::msg::Byte) -> robot_bus::std_msgs::msg::v1::Byte {
    robot_bus::std_msgs::msg::v1::Byte {
        data: u32::from(msg.data),
    }
}

pub(crate) fn byte_to_ros(bus: robot_bus::std_msgs::msg::v1::Byte) -> ros_env::std_msgs::msg::Byte {
    ros_env::std_msgs::msg::Byte {
        data: bus.data as u8,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsByteMapper;

impl TypedTopicMapper for StdMsgsByteMapper {
    type Ros = ros_env::std_msgs::msg::Byte;
    type Bus = robot_bus::std_msgs::msg::v1::Byte;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(byte_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(byte_to_ros(msg))
    }
}
