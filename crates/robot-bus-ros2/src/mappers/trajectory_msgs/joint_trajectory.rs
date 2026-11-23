//! Typed mapper for `trajectory_msgs/msg/JointTrajectory`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn joint_trajectory_to_bus(
    msg: ros_env::trajectory_msgs::msg::JointTrajectory,
) -> robot_bus::trajectory_msgs::msg::v1::JointTrajectory {
    robot_bus::trajectory_msgs::msg::v1::JointTrajectory {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::mappers::convert::string_seq(msg.joint_names),
        points: msg.points.into_iter().map(crate::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_bus).collect(),
    }
}

pub(crate) fn joint_trajectory_to_ros(
    bus: robot_bus::trajectory_msgs::msg::v1::JointTrajectory,
) -> ros_env::trajectory_msgs::msg::JointTrajectory {
    ros_env::trajectory_msgs::msg::JointTrajectory {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::mappers::convert::ros_string_seq(bus.joint_names),
        points: bus.points.into_iter().map(crate::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryMsgsJointTrajectoryMapper;

impl TypedTopicMapper for TrajectoryMsgsJointTrajectoryMapper {
    type Ros = ros_env::trajectory_msgs::msg::JointTrajectory;
    type Bus = robot_bus::trajectory_msgs::msg::v1::JointTrajectory;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(joint_trajectory_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(joint_trajectory_to_ros(msg))
    }
}
