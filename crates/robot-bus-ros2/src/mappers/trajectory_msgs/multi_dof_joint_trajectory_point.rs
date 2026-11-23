//! Typed mapper for `trajectory_msgs/msg/MultiDOFJointTrajectoryPoint`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn multi_dof_joint_trajectory_point_to_bus(
    msg: ros_env::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint,
) -> robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint {
    robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint {
        transforms: msg
            .transforms
            .into_iter()
            .map(crate::mappers::geometry_msgs::transform::transform_to_bus)
            .collect(),
        velocities: msg
            .velocities
            .into_iter()
            .map(crate::mappers::geometry_msgs::twist::twist_to_bus)
            .collect(),
        accelerations: msg
            .accelerations
            .into_iter()
            .map(crate::mappers::geometry_msgs::twist::twist_to_bus)
            .collect(),
        time_from_start: Some(
            crate::mappers::builtin_interfaces::duration::duration_to_bus(
                msg.time_from_start,
            ),
        ),
    }
}

pub(crate) fn multi_dof_joint_trajectory_point_to_ros(
    bus: robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint,
) -> ros_env::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint {
    ros_env::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint {
        transforms: bus
            .transforms
            .into_iter()
            .map(crate::mappers::geometry_msgs::transform::transform_to_ros)
            .collect(),
        velocities: bus
            .velocities
            .into_iter()
            .map(crate::mappers::geometry_msgs::twist::twist_to_ros)
            .collect(),
        accelerations: bus
            .accelerations
            .into_iter()
            .map(crate::mappers::geometry_msgs::twist::twist_to_ros)
            .collect(),
        time_from_start: crate::mappers::builtin_interfaces::duration::duration_to_ros(
            bus.time_from_start.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryMsgsMultiDofJointTrajectoryPointMapper;

impl TypedTopicMapper for TrajectoryMsgsMultiDofJointTrajectoryPointMapper {
    type Ros = ros_env::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint;
    type Bus = robot_bus::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(multi_dof_joint_trajectory_point_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(multi_dof_joint_trajectory_point_to_ros(msg))
    }
}
