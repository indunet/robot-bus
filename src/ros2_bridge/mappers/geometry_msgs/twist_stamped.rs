//! Typed mapper for `geometry_msgs/msg/TwistStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn twist_stamped_to_bus(msg: ros_env::geometry_msgs::msg::TwistStamped) -> crate::geometry_msgs::msg::v1::TwistStamped {
    crate::geometry_msgs::msg::v1::TwistStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        twist: Some(crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_bus(msg.twist)),
    }
}

pub(crate) fn twist_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::TwistStamped) -> ros_env::geometry_msgs::msg::TwistStamped {
    ros_env::geometry_msgs::msg::TwistStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        twist: crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_ros(bus.twist.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistStampedMapper;

impl TypedTopicMapper for GeometryMsgsTwistStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::TwistStamped;
    type Bus = crate::geometry_msgs::msg::v1::TwistStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/TwistStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(twist_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(twist_stamped_to_ros(msg))
    }
}
