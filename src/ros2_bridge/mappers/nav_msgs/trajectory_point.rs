//! Typed mapper for `nav_msgs/msg/TrajectoryPoint`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn trajectory_point_to_bus(msg: ros_env::nav_msgs::msg::TrajectoryPoint) -> crate::nav_msgs::msg::v1::TrajectoryPoint {
    crate::nav_msgs::msg::v1::TrajectoryPoint {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        pose: Some(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        velocity: Some(crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_bus(msg.velocity)),
        acceleration: Some(crate::ros2_bridge::mappers::geometry_msgs::accel::accel_to_bus(msg.acceleration)),
        effort: Some(crate::ros2_bridge::mappers::geometry_msgs::wrench::wrench_to_bus(msg.effort)),
    }
}

pub(crate) fn trajectory_point_to_ros(bus: crate::nav_msgs::msg::v1::TrajectoryPoint) -> ros_env::nav_msgs::msg::TrajectoryPoint {
    ros_env::nav_msgs::msg::TrajectoryPoint {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        pose: crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        velocity: crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_ros(bus.velocity.unwrap_or_default()),
        acceleration: crate::ros2_bridge::mappers::geometry_msgs::accel::accel_to_ros(bus.acceleration.unwrap_or_default()),
        effort: crate::ros2_bridge::mappers::geometry_msgs::wrench::wrench_to_ros(bus.effort.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsTrajectoryPointMapper;

impl TypedTopicMapper for NavMsgsTrajectoryPointMapper {
    type Ros = ros_env::nav_msgs::msg::TrajectoryPoint;
    type Bus = crate::nav_msgs::msg::v1::TrajectoryPoint;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(trajectory_point_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(trajectory_point_to_ros(msg))
    }
}
