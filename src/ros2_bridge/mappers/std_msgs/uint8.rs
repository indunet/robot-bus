//! Typed mapper for `std_msgs/msg/UInt8`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uint8_to_bus(msg: ros_env::std_msgs::msg::UInt8) -> crate::std_msgs::msg::v1::UInt8 {
    crate::std_msgs::msg::v1::UInt8 {
        data: u32::from(msg.data),
    }
}

pub(crate) fn uint8_to_ros(bus: crate::std_msgs::msg::v1::UInt8) -> ros_env::std_msgs::msg::UInt8 {
    ros_env::std_msgs::msg::UInt8 {
        data: bus.data as u8,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt8Mapper;

impl TypedTopicMapper for StdMsgsUInt8Mapper {
    type Ros = ros_env::std_msgs::msg::UInt8;
    type Bus = crate::std_msgs::msg::v1::UInt8;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/UInt8"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uint8_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uint8_to_ros(msg))
    }
}
