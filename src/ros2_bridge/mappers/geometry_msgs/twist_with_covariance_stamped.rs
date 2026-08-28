//! Typed mapper for `geometry_msgs/msg/TwistWithCovarianceStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn twist_with_covariance_stamped_to_bus(msg: ros_env::geometry_msgs::msg::TwistWithCovarianceStamped) -> crate::geometry_msgs::msg::v1::TwistWithCovarianceStamped {
    crate::geometry_msgs::msg::v1::TwistWithCovarianceStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        twist: Some(crate::ros2_bridge::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_bus(msg.twist)),
    }
}

pub(crate) fn twist_with_covariance_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::TwistWithCovarianceStamped) -> ros_env::geometry_msgs::msg::TwistWithCovarianceStamped {
    ros_env::geometry_msgs::msg::TwistWithCovarianceStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        twist: crate::ros2_bridge::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_ros(bus.twist.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistWithCovarianceStampedMapper;

impl TypedTopicMapper for GeometryMsgsTwistWithCovarianceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::TwistWithCovarianceStamped;
    type Bus = crate::geometry_msgs::msg::v1::TwistWithCovarianceStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/TwistWithCovarianceStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(twist_with_covariance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(twist_with_covariance_stamped_to_ros(msg))
    }
}
