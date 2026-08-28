//! Typed mapper for `std_msgs/msg/Header`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn header_to_bus(msg: ros_env::std_msgs::msg::Header) -> crate::std_msgs::msg::v1::Header {
    crate::std_msgs::msg::v1::Header {
        stamp: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.stamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
    }
}

pub(crate) fn header_to_ros(bus: crate::std_msgs::msg::v1::Header) -> ros_env::std_msgs::msg::Header {
    ros_env::std_msgs::msg::Header {
        stamp: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.stamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsHeaderMapper;

impl TypedTopicMapper for StdMsgsHeaderMapper {
    type Ros = ros_env::std_msgs::msg::Header;
    type Bus = crate::std_msgs::msg::v1::Header;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Header"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(header_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(header_to_ros(msg))
    }
}
