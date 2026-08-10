//! Mapper for `visualization_msgs/msg/MenuEntry`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn menu_entry_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::MenuEntry> {
    Ok(crate::visualization_msgs::msg::v1::MenuEntry {
        id: read_u32(view, "id")?,
        parent_id: read_u32(view, "parent_id")?,
        title: read_string(view, "title")?,
        command: read_string(view, "command")?,
        command_type: read_u32(view, "command_type")?,
    })
}

pub(crate) fn menu_entry_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::MenuEntry,
) -> Result<()> {
    write_u32(view, "id", bus.id)?;
    write_u32(view, "parent_id", bus.parent_id)?;
    write_string(view, "title", &bus.title)?;
    write_string(view, "command", &bus.command)?;
    write_u32(view, "command_type", bus.command_type)?;
    Ok(())
}

pub(crate) fn menu_entry_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::MenuEntry> {
    menu_entry_from_view(&msg.view())
}

pub(crate) fn menu_entry_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::MenuEntry,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/MenuEntry")?;
    menu_entry_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsMenuEntryMapper;
impl TopicMapper for VisualizationMsgsMenuEntryMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/MenuEntry"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(menu_entry_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::visualization_msgs::msg::v1::MenuEntry as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode visualization_msgs/msg/MenuEntry: {e}"))
            })?;
        menu_entry_bus_to_dyn(&bus)
    }
}
