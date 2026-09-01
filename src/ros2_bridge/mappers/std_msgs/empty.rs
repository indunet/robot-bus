//! Typed mapper for `std_msgs/msg/Empty`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn empty_to_bus(_msg: ros_env::std_msgs::msg::Empty) -> crate::std_msgs::msg::v1::Empty {
    crate::std_msgs::msg::v1::Empty {}
}

pub(crate) fn empty_to_ros(_bus: crate::std_msgs::msg::v1::Empty) -> ros_env::std_msgs::msg::Empty {
    ros_env::std_msgs::msg::Empty {}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsEmptyMapper;

impl TypedTopicMapper for StdMsgsEmptyMapper {
    type Ros = ros_env::std_msgs::msg::Empty;
    type Bus = crate::std_msgs::msg::v1::Empty;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(empty_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(empty_to_ros(msg))
    }
}
