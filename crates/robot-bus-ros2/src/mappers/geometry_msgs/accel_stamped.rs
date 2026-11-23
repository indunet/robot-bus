//! Typed mapper for `geometry_msgs/msg/AccelStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn accel_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::AccelStamped,
) -> robot_bus::geometry_msgs::msg::v1::AccelStamped {
    robot_bus::geometry_msgs::msg::v1::AccelStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        accel: Some(crate::mappers::geometry_msgs::accel::accel_to_bus(msg.accel)),
    }
}

pub(crate) fn accel_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::AccelStamped,
) -> ros_env::geometry_msgs::msg::AccelStamped {
    ros_env::geometry_msgs::msg::AccelStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        accel: crate::mappers::geometry_msgs::accel::accel_to_ros(
            bus.accel.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsAccelStampedMapper;

impl TypedTopicMapper for GeometryMsgsAccelStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::AccelStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::AccelStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(accel_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(accel_stamped_to_ros(msg))
    }
}
