//! Typed mapper for `trajectory_msgs/msg/JointTrajectoryPoint`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_trajectory_point_to_bus(
    msg: ros_env::trajectory_msgs::msg::JointTrajectoryPoint,
) -> crate::trajectory_msgs::msg::v1::JointTrajectoryPoint {
    crate::trajectory_msgs::msg::v1::JointTrajectoryPoint {
        positions: crate::ros2_bridge::mappers::convert::f64_seq(msg.positions),
        velocities: crate::ros2_bridge::mappers::convert::f64_seq(msg.velocities),
        accelerations: crate::ros2_bridge::mappers::convert::f64_seq(msg.accelerations),
        effort: crate::ros2_bridge::mappers::convert::f64_seq(msg.effort),
        time_from_start: Some(
            crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_bus(
                msg.time_from_start,
            ),
        ),
    }
}

pub(crate) fn joint_trajectory_point_to_ros(
    bus: crate::trajectory_msgs::msg::v1::JointTrajectoryPoint,
) -> ros_env::trajectory_msgs::msg::JointTrajectoryPoint {
    ros_env::trajectory_msgs::msg::JointTrajectoryPoint {
        positions: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.positions),
        velocities: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.velocities),
        accelerations: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(
            bus.accelerations,
        ),
        effort: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.effort),
        time_from_start: crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_ros(
            bus.time_from_start.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryMsgsJointTrajectoryPointMapper;

impl TypedTopicMapper for TrajectoryMsgsJointTrajectoryPointMapper {
    type Ros = ros_env::trajectory_msgs::msg::JointTrajectoryPoint;
    type Bus = crate::trajectory_msgs::msg::v1::JointTrajectoryPoint;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_trajectory_point_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_trajectory_point_to_ros(msg))
    }
}
