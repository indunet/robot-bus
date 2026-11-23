//! Typed mapper for `geometry_msgs/msg/Accel`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn accel_to_bus(
    msg: ros_env::geometry_msgs::msg::Accel,
) -> robot_bus::geometry_msgs::msg::v1::Accel {
    robot_bus::geometry_msgs::msg::v1::Accel {
        linear: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.linear),
        ),
        angular: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.angular),
        ),
    }
}

pub(crate) fn accel_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Accel,
) -> ros_env::geometry_msgs::msg::Accel {
    ros_env::geometry_msgs::msg::Accel {
        linear: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.linear.unwrap_or_default(),
        ),
        angular: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.angular.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsAccelMapper;

impl TypedTopicMapper for GeometryMsgsAccelMapper {
    type Ros = ros_env::geometry_msgs::msg::Accel;
    type Bus = robot_bus::geometry_msgs::msg::v1::Accel;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(accel_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(accel_to_ros(msg))
    }
}
