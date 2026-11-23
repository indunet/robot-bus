//! Typed mapper for `trajectory_msgs/msg/MultiDOFJointTrajectory`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn multi_dof_joint_trajectory_to_bus(
    msg: ros_env::trajectory_msgs::msg::MultiDOFJointTrajectory,
) -> robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectory {
    robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectory {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::mappers::convert::string_seq(msg.joint_names),
        points: msg.points.into_iter().map(crate::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_bus).collect(),
    }
}

pub(crate) fn multi_dof_joint_trajectory_to_ros(
    bus: robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectory,
) -> ros_env::trajectory_msgs::msg::MultiDOFJointTrajectory {
    ros_env::trajectory_msgs::msg::MultiDOFJointTrajectory {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::mappers::convert::ros_string_seq(bus.joint_names),
        points: bus.points.into_iter().map(crate::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryMsgsMultiDofJointTrajectoryMapper;

impl TypedTopicMapper for TrajectoryMsgsMultiDofJointTrajectoryMapper {
    type Ros = ros_env::trajectory_msgs::msg::MultiDOFJointTrajectory;
    type Bus = robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectory;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(multi_dof_joint_trajectory_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(multi_dof_joint_trajectory_to_ros(msg))
    }
}
