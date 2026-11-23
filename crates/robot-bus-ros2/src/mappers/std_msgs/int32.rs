//! Typed mapper for `std_msgs/msg/Int32`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn int32_to_bus(msg: ros_env::std_msgs::msg::Int32) -> robot_bus::std_msgs::msg::v1::Int32 {
    robot_bus::std_msgs::msg::v1::Int32 {
        data: msg.data.into(),
    }
}

pub(crate) fn int32_to_ros(bus: robot_bus::std_msgs::msg::v1::Int32) -> ros_env::std_msgs::msg::Int32 {
    ros_env::std_msgs::msg::Int32 {
        data: bus.data as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt32Mapper;

impl TypedTopicMapper for StdMsgsInt32Mapper {
    type Ros = ros_env::std_msgs::msg::Int32;
    type Bus = robot_bus::std_msgs::msg::v1::Int32;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(int32_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(int32_to_ros(msg))
    }
}
