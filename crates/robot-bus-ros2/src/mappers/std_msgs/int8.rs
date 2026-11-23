//! Typed mapper for `std_msgs/msg/Int8`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn int8_to_bus(msg: ros_env::std_msgs::msg::Int8) -> robot_bus::std_msgs::msg::v1::Int8 {
    robot_bus::std_msgs::msg::v1::Int8 {
        data: msg.data as i32,
    }
}

pub(crate) fn int8_to_ros(bus: robot_bus::std_msgs::msg::v1::Int8) -> ros_env::std_msgs::msg::Int8 {
    ros_env::std_msgs::msg::Int8 {
        data: bus.data as i8,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt8Mapper;

impl TypedTopicMapper for StdMsgsInt8Mapper {
    type Ros = ros_env::std_msgs::msg::Int8;
    type Bus = robot_bus::std_msgs::msg::v1::Int8;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(int8_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(int8_to_ros(msg))
    }
}
