//! Mapper for `control_msgs/msg/JointControllerState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joint_controller_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::JointControllerState> {
    Ok(crate::control_msgs::msg::v1::JointControllerState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        set_point: read_f64(view, "set_point")?,
        process_value: read_f64(view, "process_value")?,
        process_value_dot: read_f64(view, "process_value_dot")?,
        error: read_f64(view, "error")?,
        time_step: read_f64(view, "time_step")?,
        command: read_f64(view, "command")?,
        p: read_f64(view, "p")?,
        i: read_f64(view, "i")?,
        d: read_f64(view, "d")?,
        i_clamp: read_f64(view, "i_clamp")?,
        antiwindup: read_bool(view, "antiwindup")?,
    })
}

pub(crate) fn joint_controller_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::JointControllerState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64(view, "set_point", bus.set_point)?;
    write_f64(view, "process_value", bus.process_value)?;
    write_f64(view, "process_value_dot", bus.process_value_dot)?;
    write_f64(view, "error", bus.error)?;
    write_f64(view, "time_step", bus.time_step)?;
    write_f64(view, "command", bus.command)?;
    write_f64(view, "p", bus.p)?;
    write_f64(view, "i", bus.i)?;
    write_f64(view, "d", bus.d)?;
    write_f64(view, "i_clamp", bus.i_clamp)?;
    write_bool(view, "antiwindup", bus.antiwindup)?;
    Ok(())
}

pub(crate) fn joint_controller_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::JointControllerState> {
    joint_controller_state_from_view(&msg.view())
}

pub(crate) fn joint_controller_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::JointControllerState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/JointControllerState")?;
    joint_controller_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsJointControllerStateMapper;
impl TopicMapper for ControlMsgsJointControllerStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/JointControllerState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_controller_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::JointControllerState as ProstMessage>::decode(payload)
                .map_err(|e| {
                BusError::Protocol(format!("decode control_msgs/msg/JointControllerState: {e}"))
            })?;
        joint_controller_state_bus_to_dyn(&bus)
    }
}
