//! Typed mapper for `geometry_msgs/msg/PoseWithCovariance`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn pose_with_covariance_to_bus(
    msg: ros_env::geometry_msgs::msg::PoseWithCovariance,
) -> robot_bus::geometry_msgs::msg::v1::PoseWithCovariance {
    robot_bus::geometry_msgs::msg::v1::PoseWithCovariance {
        pose: Some(crate::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        covariance: crate::mappers::convert::f64_seq(msg.covariance),
    }
}

pub(crate) fn pose_with_covariance_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::PoseWithCovariance,
) -> ros_env::geometry_msgs::msg::PoseWithCovariance {
    ros_env::geometry_msgs::msg::PoseWithCovariance {
        pose: crate::mappers::geometry_msgs::pose::pose_to_ros(
            bus.pose.unwrap_or_default(),
        ),
        covariance: crate::mappers::convert::FromF64Seq::from_f64_seq(bus.covariance),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoseWithCovarianceMapper;

impl TypedTopicMapper for GeometryMsgsPoseWithCovarianceMapper {
    type Ros = ros_env::geometry_msgs::msg::PoseWithCovariance;
    type Bus = robot_bus::geometry_msgs::msg::v1::PoseWithCovariance;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(pose_with_covariance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(pose_with_covariance_to_ros(msg))
    }
}
