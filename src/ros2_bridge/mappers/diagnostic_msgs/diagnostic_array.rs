//! Mapper for `diagnostic_msgs/msg/DiagnosticArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn diagnostic_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::diagnostic_msgs::msg::v1::DiagnosticArray> {
    Ok(crate::diagnostic_msgs::msg::v1::DiagnosticArray {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        status: read_message_seq(
            view,
            "status",
            super::diagnostic_status::diagnostic_status_from_view,
        )?,
    })
}

pub(crate) fn diagnostic_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::diagnostic_msgs::msg::v1::DiagnosticArray,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "status",
        &bus.status,
        super::diagnostic_status::diagnostic_status_write,
    )?;
    Ok(())
}

pub(crate) fn diagnostic_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::diagnostic_msgs::msg::v1::DiagnosticArray> {
    diagnostic_array_from_view(&msg.view())
}

pub(crate) fn diagnostic_array_bus_to_dyn(
    bus: &crate::diagnostic_msgs::msg::v1::DiagnosticArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("diagnostic_msgs/msg/DiagnosticArray")?;
    diagnostic_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct DiagnosticMsgsDiagnosticArrayMapper;
impl TopicMapper for DiagnosticMsgsDiagnosticArrayMapper {
    fn type_name(&self) -> &'static str {
        "diagnostic_msgs/msg/DiagnosticArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(diagnostic_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::diagnostic_msgs::msg::v1::DiagnosticArray as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode diagnostic_msgs/msg/DiagnosticArray: {e}"))
                })?;
        diagnostic_array_bus_to_dyn(&bus)
    }
}
