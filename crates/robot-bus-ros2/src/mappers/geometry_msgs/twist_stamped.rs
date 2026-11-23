//! Typed mapper for `geometry_msgs/msg/TwistStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn twist_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::TwistStamped,
) -> robot_bus::geometry_msgs::msg::v1::TwistStamped {
    robot_bus::geometry_msgs::msg::v1::TwistStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        twist: Some(crate::mappers::geometry_msgs::twist::twist_to_bus(msg.twist)),
    }
}

pub(crate) fn twist_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::TwistStamped,
) -> ros_env::geometry_msgs::msg::TwistStamped {
    ros_env::geometry_msgs::msg::TwistStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        twist: crate::mappers::geometry_msgs::twist::twist_to_ros(
            bus.twist.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistStampedMapper;

impl TypedTopicMapper for GeometryMsgsTwistStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::TwistStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::TwistStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(twist_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(twist_stamped_to_ros(msg))
    }
}
