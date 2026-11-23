//! Typed mapper for `geometry_msgs/msg/Accel`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn accel_to_bus(
    msg: ros_env::geometry_msgs::msg::Accel,
) -> crate::geometry_msgs::msg::v1::Accel {
    crate::geometry_msgs::msg::v1::Accel {
        linear: Some(
            crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.linear),
        ),
        angular: Some(
            crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.angular),
        ),
    }
}

pub(crate) fn accel_to_ros(
    bus: crate::geometry_msgs::msg::v1::Accel,
) -> ros_env::geometry_msgs::msg::Accel {
    ros_env::geometry_msgs::msg::Accel {
        linear: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.linear.unwrap_or_default(),
        ),
        angular: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.angular.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsAccelMapper;

impl TypedTopicMapper for GeometryMsgsAccelMapper {
    type Ros = ros_env::geometry_msgs::msg::Accel;
    type Bus = crate::geometry_msgs::msg::v1::Accel;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(accel_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(accel_to_ros(msg))
    }
}
