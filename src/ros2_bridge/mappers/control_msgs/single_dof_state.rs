//! Mapper for `control_msgs/msg/SingleDOFState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn single_dof_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::SingleDofState> {
    Ok(crate::control_msgs::msg::v1::SingleDofState {
        name: read_string(view, "name")?,
        reference: read_f64(view, "reference")?,
        feedback: read_f64(view, "feedback")?,
        feedback_dot: read_f64(view, "feedback_dot")?,
        error: read_f64(view, "error")?,
        error_dot: read_f64(view, "error_dot")?,
        time_step: read_f64(view, "time_step")?,
        output: read_f64(view, "output")?,
    })
}

pub(crate) fn single_dof_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::SingleDofState,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    write_f64(view, "reference", bus.reference)?;
    write_f64(view, "feedback", bus.feedback)?;
    write_f64(view, "feedback_dot", bus.feedback_dot)?;
    write_f64(view, "error", bus.error)?;
    write_f64(view, "error_dot", bus.error_dot)?;
    write_f64(view, "time_step", bus.time_step)?;
    write_f64(view, "output", bus.output)?;
    Ok(())
}

pub(crate) fn single_dof_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::SingleDofState> {
    single_dof_state_from_view(&msg.view())
}

pub(crate) fn single_dof_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::SingleDofState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/SingleDOFState")?;
    single_dof_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsSingleDofStateMapper;
impl TopicMapper for ControlMsgsSingleDofStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/SingleDOFState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(single_dof_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::SingleDofState as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode control_msgs/msg/SingleDOFState: {e}"))
        })?;
        single_dof_state_bus_to_dyn(&bus)
    }
}
