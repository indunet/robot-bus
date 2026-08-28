//! Typed mapper for `geometry_msgs/msg/PoseWithCovarianceStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn pose_with_covariance_stamped_to_bus(msg: ros_env::geometry_msgs::msg::PoseWithCovarianceStamped) -> crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped {
    crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        pose: Some(crate::ros2_bridge::mappers::geometry_msgs::pose_with_covariance::pose_with_covariance_to_bus(msg.pose)),
    }
}

pub(crate) fn pose_with_covariance_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped) -> ros_env::geometry_msgs::msg::PoseWithCovarianceStamped {
    ros_env::geometry_msgs::msg::PoseWithCovarianceStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        pose: crate::ros2_bridge::mappers::geometry_msgs::pose_with_covariance::pose_with_covariance_to_ros(bus.pose.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoseWithCovarianceStampedMapper;

impl TypedTopicMapper for GeometryMsgsPoseWithCovarianceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PoseWithCovarianceStamped;
    type Bus = crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PoseWithCovarianceStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(pose_with_covariance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(pose_with_covariance_stamped_to_ros(msg))
    }
}
