//! Typed mapper for `geometry_msgs/msg/QuaternionStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn quaternion_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::QuaternionStamped,
) -> robot_bus::geometry_msgs::msg::v1::QuaternionStamped {
    robot_bus::geometry_msgs::msg::v1::QuaternionStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        quaternion: Some(
            crate::mappers::geometry_msgs::quaternion::quaternion_to_bus(
                msg.quaternion,
            ),
        ),
    }
}

pub(crate) fn quaternion_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::QuaternionStamped,
) -> ros_env::geometry_msgs::msg::QuaternionStamped {
    ros_env::geometry_msgs::msg::QuaternionStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        quaternion: crate::mappers::geometry_msgs::quaternion::quaternion_to_ros(
            bus.quaternion.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsQuaternionStampedMapper;

impl TypedTopicMapper for GeometryMsgsQuaternionStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::QuaternionStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::QuaternionStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(quaternion_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(quaternion_stamped_to_ros(msg))
    }
}
