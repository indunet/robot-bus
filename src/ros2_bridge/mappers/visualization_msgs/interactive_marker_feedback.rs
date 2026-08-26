//! Mapper for `visualization_msgs/msg/InteractiveMarkerFeedback`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn interactive_marker_feedback_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerFeedback> {
    Ok(
        crate::visualization_msgs::msg::v1::InteractiveMarkerFeedback {
            header: nested_view(view, "header")?
                .as_ref()
                .map(super::super::std_msgs::header::header_from_view)
                .transpose()?,
            client_id: read_string(view, "client_id")?,
            marker_name: read_string(view, "marker_name")?,
            control_name: read_string(view, "control_name")?,
            event_type: read_u32(view, "event_type")?,
            pose: nested_view(view, "pose")?
                .as_ref()
                .map(super::super::geometry_msgs::pose::pose_from_view)
                .transpose()?,
            menu_entry_id: read_u32(view, "menu_entry_id")?,
            mouse_point: nested_view(view, "mouse_point")?
                .as_ref()
                .map(super::super::geometry_msgs::point::point_from_view)
                .transpose()?,
            mouse_point_valid: read_bool(view, "mouse_point_valid")?,
        },
    )
}

pub(crate) fn interactive_marker_feedback_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerFeedback,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string(view, "client_id", &bus.client_id)?;
    write_string(view, "marker_name", &bus.marker_name)?;
    write_string(view, "control_name", &bus.control_name)?;
    write_u32(view, "event_type", bus.event_type)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    write_u32(view, "menu_entry_id", bus.menu_entry_id)?;
    if let Some(v) = &bus.mouse_point {
        with_nested_mut(view, "mouse_point", |nested| {
            super::super::geometry_msgs::point::point_write(nested, v)
        })?;
    }
    write_bool(view, "mouse_point_valid", bus.mouse_point_valid)?;
    Ok(())
}

pub(crate) fn interactive_marker_feedback_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerFeedback> {
    interactive_marker_feedback_from_view(&msg.view())
}

pub(crate) fn interactive_marker_feedback_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerFeedback,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/InteractiveMarkerFeedback")?;
    interactive_marker_feedback_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsInteractiveMarkerFeedbackMapper;
impl TopicMapper for VisualizationMsgsInteractiveMarkerFeedbackMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/InteractiveMarkerFeedback"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interactive_marker_feedback_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::visualization_msgs::msg::v1::InteractiveMarkerFeedback as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode visualization_msgs/msg/InteractiveMarkerFeedback: {e}"))
            })?;
        interactive_marker_feedback_bus_to_dyn(&bus)
    }
}
