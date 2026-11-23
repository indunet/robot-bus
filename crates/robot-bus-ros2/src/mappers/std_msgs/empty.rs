//! Typed mapper for `std_msgs/msg/Empty`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn empty_to_bus(_msg: ros_env::std_msgs::msg::Empty) -> robot_bus::std_msgs::msg::v1::Empty {
    robot_bus::std_msgs::msg::v1::Empty {}
}

pub(crate) fn empty_to_ros(_bus: robot_bus::std_msgs::msg::v1::Empty) -> ros_env::std_msgs::msg::Empty {
    // Distro IDL uses `structure_needs_at_least_one_member`; Default covers both.
    ros_env::std_msgs::msg::Empty::default()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsEmptyMapper;

impl TypedTopicMapper for StdMsgsEmptyMapper {
    type Ros = ros_env::std_msgs::msg::Empty;
    type Bus = robot_bus::std_msgs::msg::v1::Empty;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(empty_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(empty_to_ros(msg))
    }
}
