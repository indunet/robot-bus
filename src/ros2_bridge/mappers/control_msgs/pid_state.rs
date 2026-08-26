//! Mapper for `control_msgs/msg/PidState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn pid_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::PidState> {
    Ok(crate::control_msgs::msg::v1::PidState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        timestep: nested_view(view, "timestep")?
            .as_ref()
            .map(super::super::builtin_interfaces::duration::duration_from_view)
            .transpose()?,
        error: read_f64(view, "error")?,
        error_dot: read_f64(view, "error_dot")?,
        p_error: read_f64(view, "p_error")?,
        i_error: read_f64(view, "i_error")?,
        d_error: read_f64(view, "d_error")?,
        p_term: read_f64(view, "p_term")?,
        i_term: read_f64(view, "i_term")?,
        d_term: read_f64(view, "d_term")?,
        i_max: read_f64(view, "i_max")?,
        i_min: read_f64(view, "i_min")?,
        output: read_f64(view, "output")?,
    })
}

pub(crate) fn pid_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::PidState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.timestep {
        with_nested_mut(view, "timestep", |nested| {
            super::super::builtin_interfaces::duration::duration_write(nested, v)
        })?;
    }
    write_f64(view, "error", bus.error)?;
    write_f64(view, "error_dot", bus.error_dot)?;
    write_f64(view, "p_error", bus.p_error)?;
    write_f64(view, "i_error", bus.i_error)?;
    write_f64(view, "d_error", bus.d_error)?;
    write_f64(view, "p_term", bus.p_term)?;
    write_f64(view, "i_term", bus.i_term)?;
    write_f64(view, "d_term", bus.d_term)?;
    write_f64(view, "i_max", bus.i_max)?;
    write_f64(view, "i_min", bus.i_min)?;
    write_f64(view, "output", bus.output)?;
    Ok(())
}

pub(crate) fn pid_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::PidState> {
    pid_state_from_view(&msg.view())
}

pub(crate) fn pid_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::PidState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/PidState")?;
    pid_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsPidStateMapper;
impl TopicMapper for ControlMsgsPidStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/PidState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(pid_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::PidState as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode control_msgs/msg/PidState: {e}")))?;
        pid_state_bus_to_dyn(&bus)
    }
}
