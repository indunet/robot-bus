//! Typed mapper for `geometry_msgs/msg/TwistWithCovariance`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn twist_with_covariance_to_bus(
    msg: ros_env::geometry_msgs::msg::TwistWithCovariance,
) -> robot_bus::geometry_msgs::msg::v1::TwistWithCovariance {
    robot_bus::geometry_msgs::msg::v1::TwistWithCovariance {
        twist: Some(crate::mappers::geometry_msgs::twist::twist_to_bus(msg.twist)),
        covariance: crate::mappers::convert::f64_seq(msg.covariance),
    }
}

pub(crate) fn twist_with_covariance_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::TwistWithCovariance,
) -> ros_env::geometry_msgs::msg::TwistWithCovariance {
    ros_env::geometry_msgs::msg::TwistWithCovariance {
        twist: crate::mappers::geometry_msgs::twist::twist_to_ros(
            bus.twist.unwrap_or_default(),
        ),
        covariance: crate::mappers::convert::FromF64Seq::from_f64_seq(bus.covariance),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTwistWithCovarianceMapper;

impl TypedTopicMapper for GeometryMsgsTwistWithCovarianceMapper {
    type Ros = ros_env::geometry_msgs::msg::TwistWithCovariance;
    type Bus = robot_bus::geometry_msgs::msg::v1::TwistWithCovariance;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(twist_with_covariance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(twist_with_covariance_to_ros(msg))
    }
}
