//! Typed mapper for `control_msgs/msg/JointTrajectoryControllerState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_trajectory_controller_state_to_bus(msg: ros_env::control_msgs::msg::JointTrajectoryControllerState) -> crate::control_msgs::msg::v1::JointTrajectoryControllerState {
    crate::control_msgs::msg::v1::JointTrajectoryControllerState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.joint_names),
        reference: Some(crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_bus(msg.reference)),
        feedback: Some(crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_bus(msg.feedback)),
        error: Some(crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_bus(msg.error)),
        output: Some(crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_bus(msg.output)),
        multi_dof_joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.multi_dof_joint_names),
        multi_dof_reference: Some(crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_reference)),
        multi_dof_feedback: Some(crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_feedback)),
        multi_dof_error: Some(crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_error)),
        multi_dof_output: Some(crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_output)),
    }
}

pub(crate) fn joint_trajectory_controller_state_to_ros(bus: crate::control_msgs::msg::v1::JointTrajectoryControllerState) -> ros_env::control_msgs::msg::JointTrajectoryControllerState {
    ros_env::control_msgs::msg::JointTrajectoryControllerState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.joint_names),
        reference: crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_ros(bus.reference.unwrap_or_default()),
        feedback: crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_ros(bus.feedback.unwrap_or_default()),
        error: crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_ros(bus.error.unwrap_or_default()),
        output: crate::ros2_bridge::mappers::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_to_ros(bus.output.unwrap_or_default()),
        multi_dof_joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.multi_dof_joint_names),
        multi_dof_reference: crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_reference.unwrap_or_default()),
        multi_dof_feedback: crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_feedback.unwrap_or_default()),
        multi_dof_error: crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_error.unwrap_or_default()),
        multi_dof_output: crate::ros2_bridge::mappers::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_output.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsJointTrajectoryControllerStateMapper;

impl TypedTopicMapper for ControlMsgsJointTrajectoryControllerStateMapper {
    type Ros = ros_env::control_msgs::msg::JointTrajectoryControllerState;
    type Bus = crate::control_msgs::msg::v1::JointTrajectoryControllerState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_trajectory_controller_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_trajectory_controller_state_to_ros(msg))
    }
}
