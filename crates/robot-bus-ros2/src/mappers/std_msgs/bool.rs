//! Typed mapper for `std_msgs/msg/Bool`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn bool_to_bus(msg: ros_env::std_msgs::msg::Bool) -> robot_bus::std_msgs::msg::v1::Bool {
    robot_bus::std_msgs::msg::v1::Bool { data: msg.data }
}

pub(crate) fn bool_to_ros(bus: robot_bus::std_msgs::msg::v1::Bool) -> ros_env::std_msgs::msg::Bool {
    ros_env::std_msgs::msg::Bool { data: bus.data }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsBoolMapper;

impl TypedTopicMapper for StdMsgsBoolMapper {
    type Ros = ros_env::std_msgs::msg::Bool;
    type Bus = robot_bus::std_msgs::msg::v1::Bool;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(bool_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(bool_to_ros(msg))
    }
}
