//! Typed mapper for `std_msgs/msg/Header`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn header_to_bus(
    msg: ros_env::std_msgs::msg::Header,
) -> robot_bus::std_msgs::msg::v1::Header {
    robot_bus::std_msgs::msg::v1::Header {
        stamp: Some(crate::mappers::builtin_interfaces::time::time_to_bus(msg.stamp)),
        frame_id: crate::mappers::convert::from_ros_string(msg.frame_id),
    }
}

pub(crate) fn header_to_ros(
    bus: robot_bus::std_msgs::msg::v1::Header,
) -> ros_env::std_msgs::msg::Header {
    ros_env::std_msgs::msg::Header {
        stamp: crate::mappers::builtin_interfaces::time::time_to_ros(
            bus.stamp.unwrap_or_default(),
        ),
        frame_id: crate::mappers::convert::to_ros_string(bus.frame_id),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsHeaderMapper;

impl TypedTopicMapper for StdMsgsHeaderMapper {
    type Ros = ros_env::std_msgs::msg::Header;
    type Bus = robot_bus::std_msgs::msg::v1::Header;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(header_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(header_to_ros(msg))
    }
}
