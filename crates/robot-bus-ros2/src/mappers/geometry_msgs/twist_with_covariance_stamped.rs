//! Typed mapper for `geometry_msgs/msg/TwistWithCovarianceStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn twist_with_covariance_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::TwistWithCovarianceStamped,
) -> robot_bus::geometry_msgs::msg::v1::TwistWithCovarianceStamped {
    robot_bus::geometry_msgs::msg::v1::TwistWithCovarianceStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        twist: Some(crate::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_bus(msg.twist)),
    }
}

pub(crate) fn twist_with_covariance_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::TwistWithCovarianceStamped,
) -> ros_env::geometry_msgs::msg::TwistWithCovarianceStamped {
    ros_env::geometry_msgs::msg::TwistWithCovarianceStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        twist: crate::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_ros(bus.twist.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistWithCovarianceStampedMapper;

impl TypedTopicMapper for GeometryMsgsTwistWithCovarianceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::TwistWithCovarianceStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::TwistWithCovarianceStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(twist_with_covariance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(twist_with_covariance_stamped_to_ros(msg))
    }
}
