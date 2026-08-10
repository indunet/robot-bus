//! Mapper for `diagnostic_msgs/msg/DiagnosticStatus`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn diagnostic_status_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::diagnostic_msgs::msg::v1::DiagnosticStatus> {
    Ok(crate::diagnostic_msgs::msg::v1::DiagnosticStatus {
        level: read_u32(view, "level")?,
        name: read_string(view, "name")?,
        message: read_string(view, "message")?,
        hardware_id: read_string(view, "hardware_id")?,
        values: read_message_seq(view, "values", super::key_value::key_value_from_view)?,
    })
}

pub(crate) fn diagnostic_status_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::diagnostic_msgs::msg::v1::DiagnosticStatus,
) -> Result<()> {
    write_u32(view, "level", bus.level)?;
    write_string(view, "name", &bus.name)?;
    write_string(view, "message", &bus.message)?;
    write_string(view, "hardware_id", &bus.hardware_id)?;
    write_message_seq(view, "values", &bus.values, super::key_value::key_value_write)?;
    Ok(())
}

pub(crate) fn diagnostic_status_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::diagnostic_msgs::msg::v1::DiagnosticStatus> {
    diagnostic_status_from_view(&msg.view())
}

pub(crate) fn diagnostic_status_bus_to_dyn(
    bus: &crate::diagnostic_msgs::msg::v1::DiagnosticStatus,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("diagnostic_msgs/msg/DiagnosticStatus")?;
    diagnostic_status_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct DiagnosticMsgsDiagnosticStatusMapper;
impl TopicMapper for DiagnosticMsgsDiagnosticStatusMapper {
    fn type_name(&self) -> &'static str {
        "diagnostic_msgs/msg/DiagnosticStatus"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(diagnostic_status_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::diagnostic_msgs::msg::v1::DiagnosticStatus as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode diagnostic_msgs/msg/DiagnosticStatus: {e}"))
                })?;
        diagnostic_status_bus_to_dyn(&bus)
    }
}
