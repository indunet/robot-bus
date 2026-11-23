//! Typed mapper for `std_msgs/msg/Int16`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn int16_to_bus(msg: ros_env::std_msgs::msg::Int16) -> robot_bus::std_msgs::msg::v1::Int16 {
    robot_bus::std_msgs::msg::v1::Int16 {
        data: msg.data as i32,
    }
}

pub(crate) fn int16_to_ros(bus: robot_bus::std_msgs::msg::v1::Int16) -> ros_env::std_msgs::msg::Int16 {
    ros_env::std_msgs::msg::Int16 {
        data: bus.data as i16,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt16Mapper;

impl TypedTopicMapper for StdMsgsInt16Mapper {
    type Ros = ros_env::std_msgs::msg::Int16;
    type Bus = robot_bus::std_msgs::msg::v1::Int16;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(int16_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(int16_to_ros(msg))
    }
}
