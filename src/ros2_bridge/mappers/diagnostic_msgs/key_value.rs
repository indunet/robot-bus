//! Mapper for `diagnostic_msgs/msg/KeyValue`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn key_value_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::diagnostic_msgs::msg::v1::KeyValue> {
    Ok(crate::diagnostic_msgs::msg::v1::KeyValue {
        key: read_string(view, "key")?,
        value: read_string(view, "value")?,
    })
}

pub(crate) fn key_value_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::diagnostic_msgs::msg::v1::KeyValue,
) -> Result<()> {
    write_string(view, "key", &bus.key)?;
    write_string(view, "value", &bus.value)?;
    Ok(())
}

pub(crate) fn key_value_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::diagnostic_msgs::msg::v1::KeyValue> {
    key_value_from_view(&msg.view())
}

pub(crate) fn key_value_bus_to_dyn(
    bus: &crate::diagnostic_msgs::msg::v1::KeyValue,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("diagnostic_msgs/msg/KeyValue")?;
    key_value_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct DiagnosticMsgsKeyValueMapper;
impl TopicMapper for DiagnosticMsgsKeyValueMapper {
    fn type_name(&self) -> &'static str {
        "diagnostic_msgs/msg/KeyValue"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(key_value_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::diagnostic_msgs::msg::v1::KeyValue as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode diagnostic_msgs/msg/KeyValue: {e}")))?;
        key_value_bus_to_dyn(&bus)
    }
}
