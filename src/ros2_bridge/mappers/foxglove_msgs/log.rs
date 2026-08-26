//! Mapper for `foxglove_msgs/msg/Log`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn log_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Log> {
    Ok(crate::foxglove_msgs::msg::v1::Log {
        timestamp: read_timestamp(view, "timestamp")?,
        level: read_i32(view, "level")?,
        message: read_string(view, "message")?,
        name: read_string(view, "name")?,
        file: read_string(view, "file")?,
        line: read_u32(view, "line")?,
    })
}

pub(crate) fn log_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Log,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_i32(view, "level", bus.level)?;
    write_string(view, "message", &bus.message)?;
    write_string(view, "name", &bus.name)?;
    write_string(view, "file", &bus.file)?;
    write_u32(view, "line", bus.line)?;
    Ok(())
}

pub(crate) fn log_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Log> {
    log_from_view(&msg.view())
}

pub(crate) fn log_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Log,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Log")?;
    log_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsLogMapper;
impl TopicMapper for FoxgloveMsgsLogMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Log"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(log_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Log as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Log: {e}")))?;
        log_bus_to_dyn(&bus)
    }
}
