//! Typed mapper for `std_msgs/msg/UInt32`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uint32_to_bus(msg: ros_env::std_msgs::msg::UInt32) -> crate::std_msgs::msg::v1::UInt32 {
    crate::std_msgs::msg::v1::UInt32 {
        data: msg.data,
    }
}

pub(crate) fn uint32_to_ros(bus: crate::std_msgs::msg::v1::UInt32) -> ros_env::std_msgs::msg::UInt32 {
    ros_env::std_msgs::msg::UInt32 {
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt32Mapper;

impl TypedTopicMapper for StdMsgsUInt32Mapper {
    type Ros = ros_env::std_msgs::msg::UInt32;
    type Bus = crate::std_msgs::msg::v1::UInt32;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/UInt32"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uint32_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uint32_to_ros(msg))
    }
}
