//! Typed mapper for `geometry_msgs/msg/Twist`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn twist_to_bus(msg: ros_env::geometry_msgs::msg::Twist) -> crate::geometry_msgs::msg::v1::Twist {
    crate::geometry_msgs::msg::v1::Twist {
        linear: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.linear)),
        angular: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.angular)),
    }
}

pub(crate) fn twist_to_ros(bus: crate::geometry_msgs::msg::v1::Twist) -> ros_env::geometry_msgs::msg::Twist {
    ros_env::geometry_msgs::msg::Twist {
        linear: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.linear.unwrap_or_default()),
        angular: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.angular.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistMapper;

impl TypedTopicMapper for GeometryMsgsTwistMapper {
    type Ros = ros_env::geometry_msgs::msg::Twist;
    type Bus = crate::geometry_msgs::msg::v1::Twist;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Twist"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(twist_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(twist_to_ros(msg))
    }
}
