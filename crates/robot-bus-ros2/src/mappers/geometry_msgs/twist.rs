//! Typed mapper for `geometry_msgs/msg/Twist`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn twist_to_bus(
    msg: ros_env::geometry_msgs::msg::Twist,
) -> robot_bus::geometry_msgs::msg::v1::Twist {
    robot_bus::geometry_msgs::msg::v1::Twist {
        linear: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.linear),
        ),
        angular: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.angular),
        ),
    }
}

pub(crate) fn twist_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Twist,
) -> ros_env::geometry_msgs::msg::Twist {
    ros_env::geometry_msgs::msg::Twist {
        linear: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.linear.unwrap_or_default(),
        ),
        angular: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.angular.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistMapper;

impl TypedTopicMapper for GeometryMsgsTwistMapper {
    type Ros = ros_env::geometry_msgs::msg::Twist;
    type Bus = robot_bus::geometry_msgs::msg::v1::Twist;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(twist_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(twist_to_ros(msg))
    }
}
