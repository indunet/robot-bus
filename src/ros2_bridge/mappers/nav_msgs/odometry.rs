//! Typed mapper for `nav_msgs/msg/Odometry`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn odometry_to_bus(msg: ros_env::nav_msgs::msg::Odometry) -> crate::nav_msgs::msg::v1::Odometry {
    crate::nav_msgs::msg::v1::Odometry {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        child_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.child_frame_id),
        pose: Some(crate::ros2_bridge::mappers::geometry_msgs::pose_with_covariance::pose_with_covariance_to_bus(msg.pose)),
        twist: Some(crate::ros2_bridge::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_bus(msg.twist)),
    }
}

pub(crate) fn odometry_to_ros(bus: crate::nav_msgs::msg::v1::Odometry) -> ros_env::nav_msgs::msg::Odometry {
    ros_env::nav_msgs::msg::Odometry {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        child_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.child_frame_id),
        pose: crate::ros2_bridge::mappers::geometry_msgs::pose_with_covariance::pose_with_covariance_to_ros(bus.pose.unwrap_or_default()),
        twist: crate::ros2_bridge::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_ros(bus.twist.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsOdometryMapper;

impl TypedTopicMapper for NavMsgsOdometryMapper {
    type Ros = ros_env::nav_msgs::msg::Odometry;
    type Bus = crate::nav_msgs::msg::v1::Odometry;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(odometry_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(odometry_to_ros(msg))
    }
}
