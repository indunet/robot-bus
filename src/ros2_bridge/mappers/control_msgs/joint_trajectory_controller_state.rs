//! Mapper for `control_msgs/msg/JointTrajectoryControllerState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn joint_trajectory_controller_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::JointTrajectoryControllerState> {
    Ok(
        crate::control_msgs::msg::v1::JointTrajectoryControllerState {
            header: nested_view(view, "header")?
                .as_ref()
                .map(super::super::std_msgs::header::header_from_view)
                .transpose()?,
            joint_names: read_string_seq(view, "joint_names")?,
            reference: nested_view(view, "reference")?
                .as_ref()
                .map(super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_from_view)
                .transpose()?,
            feedback: nested_view(view, "feedback")?
                .as_ref()
                .map(super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_from_view)
                .transpose()?,
            error: nested_view(view, "error")?
                .as_ref()
                .map(super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_from_view)
                .transpose()?,
            output: nested_view(view, "output")?
                .as_ref()
                .map(super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_from_view)
                .transpose()?,
            multi_dof_joint_names: read_string_seq(view, "multi_dof_joint_names")?,
            multi_dof_reference: nested_view(view, "multi_dof_reference")?
                .as_ref()
                .map(super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_from_view)
                .transpose()?,
            multi_dof_feedback: nested_view(view, "multi_dof_feedback")?
                .as_ref()
                .map(super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_from_view)
                .transpose()?,
            multi_dof_error: nested_view(view, "multi_dof_error")?
                .as_ref()
                .map(super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_from_view)
                .transpose()?,
            multi_dof_output: nested_view(view, "multi_dof_output")?
                .as_ref()
                .map(super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_from_view)
                .transpose()?,
        },
    )
}

pub(crate) fn joint_trajectory_controller_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::JointTrajectoryControllerState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string_seq(view, "joint_names", &bus.joint_names)?;
    if let Some(v) = &bus.reference {
        with_nested_mut(view, "reference", |nested| {
            super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_write(
                nested, v,
            )
        })?;
    }
    if let Some(v) = &bus.feedback {
        with_nested_mut(view, "feedback", |nested| {
            super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_write(
                nested, v,
            )
        })?;
    }
    if let Some(v) = &bus.error {
        with_nested_mut(view, "error", |nested| {
            super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_write(
                nested, v,
            )
        })?;
    }
    if let Some(v) = &bus.output {
        with_nested_mut(view, "output", |nested| {
            super::super::trajectory_msgs::joint_trajectory_point::joint_trajectory_point_write(
                nested, v,
            )
        })?;
    }
    write_string_seq(view, "multi_dof_joint_names", &bus.multi_dof_joint_names)?;
    if let Some(v) = &bus.multi_dof_reference {
        with_nested_mut(view, "multi_dof_reference", |nested| {
            super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.multi_dof_feedback {
        with_nested_mut(view, "multi_dof_feedback", |nested| {
            super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.multi_dof_error {
        with_nested_mut(view, "multi_dof_error", |nested| {
            super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.multi_dof_output {
        with_nested_mut(view, "multi_dof_output", |nested| {
            super::super::trajectory_msgs::multi_dof_joint_trajectory_point::multi_dof_joint_trajectory_point_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn joint_trajectory_controller_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::JointTrajectoryControllerState> {
    joint_trajectory_controller_state_from_view(&msg.view())
}

pub(crate) fn joint_trajectory_controller_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::JointTrajectoryControllerState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/JointTrajectoryControllerState")?;
    joint_trajectory_controller_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsJointTrajectoryControllerStateMapper;
impl TopicMapper for ControlMsgsJointTrajectoryControllerStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/JointTrajectoryControllerState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_trajectory_controller_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::JointTrajectoryControllerState as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode control_msgs/msg/JointTrajectoryControllerState: {e}"
                ))
            })?;
        joint_trajectory_controller_state_bus_to_dyn(&bus)
    }
}
