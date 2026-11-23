//! Typed mapper for `geometry_msgs/msg/AccelWithCovarianceStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn accel_with_covariance_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::AccelWithCovarianceStamped,
) -> robot_bus::geometry_msgs::msg::v1::AccelWithCovarianceStamped {
    robot_bus::geometry_msgs::msg::v1::AccelWithCovarianceStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        accel: Some(crate::mappers::geometry_msgs::accel_with_covariance::accel_with_covariance_to_bus(msg.accel)),
    }
}

pub(crate) fn accel_with_covariance_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::AccelWithCovarianceStamped,
) -> ros_env::geometry_msgs::msg::AccelWithCovarianceStamped {
    ros_env::geometry_msgs::msg::AccelWithCovarianceStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        accel: crate::mappers::geometry_msgs::accel_with_covariance::accel_with_covariance_to_ros(bus.accel.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsAccelWithCovarianceStampedMapper;

impl TypedTopicMapper for GeometryMsgsAccelWithCovarianceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::AccelWithCovarianceStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::AccelWithCovarianceStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(accel_with_covariance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(accel_with_covariance_stamped_to_ros(msg))
    }
}
