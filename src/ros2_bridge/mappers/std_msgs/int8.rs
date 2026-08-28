//! Typed mapper for `std_msgs/msg/Int8`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn int8_to_bus(msg: ros_env::std_msgs::msg::Int8) -> crate::std_msgs::msg::v1::Int8 {
    crate::std_msgs::msg::v1::Int8 {
        data: msg.data as i32,
    }
}

pub(crate) fn int8_to_ros(bus: crate::std_msgs::msg::v1::Int8) -> ros_env::std_msgs::msg::Int8 {
    ros_env::std_msgs::msg::Int8 {
        data: bus.data as i8,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt8Mapper;

impl TypedTopicMapper for StdMsgsInt8Mapper {
    type Ros = ros_env::std_msgs::msg::Int8;
    type Bus = crate::std_msgs::msg::v1::Int8;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int8"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(int8_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(int8_to_ros(msg))
    }
}
