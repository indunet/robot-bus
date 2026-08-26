//! Mapper for `visualization_msgs/msg/InteractiveMarker`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn interactive_marker_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarker> {
    Ok(crate::visualization_msgs::msg::v1::InteractiveMarker {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
        name: read_string(view, "name")?,
        description: read_string(view, "description")?,
        scale: read_f32(view, "scale")?,
        menu_entries: read_message_seq(
            view,
            "menu_entries",
            super::menu_entry::menu_entry_from_view,
        )?,
        controls: read_message_seq(
            view,
            "controls",
            super::interactive_marker_control::interactive_marker_control_from_view,
        )?,
    })
}

pub(crate) fn interactive_marker_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarker,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    write_string(view, "name", &bus.name)?;
    write_string(view, "description", &bus.description)?;
    write_f32(view, "scale", bus.scale)?;
    write_message_seq(
        view,
        "menu_entries",
        &bus.menu_entries,
        super::menu_entry::menu_entry_write,
    )?;
    write_message_seq(
        view,
        "controls",
        &bus.controls,
        super::interactive_marker_control::interactive_marker_control_write,
    )?;
    Ok(())
}

pub(crate) fn interactive_marker_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarker> {
    interactive_marker_from_view(&msg.view())
}

pub(crate) fn interactive_marker_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarker,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/InteractiveMarker")?;
    interactive_marker_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsInteractiveMarkerMapper;
impl TopicMapper for VisualizationMsgsInteractiveMarkerMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/InteractiveMarker"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interactive_marker_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::visualization_msgs::msg::v1::InteractiveMarker as ProstMessage>::decode(
            payload,
        )
        .map_err(|e| {
            BusError::Protocol(format!(
                "decode visualization_msgs/msg/InteractiveMarker: {e}"
            ))
        })?;
        interactive_marker_bus_to_dyn(&bus)
    }
}
