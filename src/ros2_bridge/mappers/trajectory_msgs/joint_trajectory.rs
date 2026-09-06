//! Typed mapper for `trajectory_msgs/msg/JointTrajectory`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_trajectory_to_bus(
    msg: ros_env::trajectory_msgs::msg::JointTrajectory,
) -> crate::trajectory_msgs::msg::v1::JointTrajectory {
    crate::trajectory_msgs::msg::v1::JointTrajectory {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.joint_names),
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_bus).collect(),
    }
}

pub(crate) fn joint_trajectory_to_ros(
    bus: crate::trajectory_msgs::msg::v1::JointTrajectory,
) -> ros_env::trajectory_msgs::msg::JointTrajectory {
    ros_env::trajectory_msgs::msg::JointTrajectory {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.joint_names),
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryMsgsJointTrajectoryMapper;

impl TypedTopicMapper for TrajectoryMsgsJointTrajectoryMapper {
    type Ros = ros_env::trajectory_msgs::msg::JointTrajectory;
    type Bus = crate::trajectory_msgs::msg::v1::JointTrajectory;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_trajectory_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_trajectory_to_ros(msg))
    }
}
