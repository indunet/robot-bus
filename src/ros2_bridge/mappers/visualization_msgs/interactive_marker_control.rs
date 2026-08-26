//! Mapper for `visualization_msgs/msg/InteractiveMarkerControl`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn interactive_marker_control_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerControl> {
    Ok(
        crate::visualization_msgs::msg::v1::InteractiveMarkerControl {
            name: read_string(view, "name")?,
            orientation: nested_view(view, "orientation")?
                .as_ref()
                .map(super::super::geometry_msgs::quaternion::quaternion_from_view)
                .transpose()?,
            orientation_mode: read_u32(view, "orientation_mode")?,
            interaction_mode: read_u32(view, "interaction_mode")?,
            always_visible: read_bool(view, "always_visible")?,
            markers: read_message_seq(view, "markers", super::marker::marker_from_view)?,
            independent_marker_orientation: read_bool(view, "independent_marker_orientation")?,
            description: read_string(view, "description")?,
        },
    )
}

pub(crate) fn interactive_marker_control_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerControl,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    if let Some(v) = &bus.orientation {
        with_nested_mut(view, "orientation", |nested| {
            super::super::geometry_msgs::quaternion::quaternion_write(nested, v)
        })?;
    }
    write_u32(view, "orientation_mode", bus.orientation_mode)?;
    write_u32(view, "interaction_mode", bus.interaction_mode)?;
    write_bool(view, "always_visible", bus.always_visible)?;
    write_message_seq(view, "markers", &bus.markers, super::marker::marker_write)?;
    write_bool(
        view,
        "independent_marker_orientation",
        bus.independent_marker_orientation,
    )?;
    write_string(view, "description", &bus.description)?;
    Ok(())
}

pub(crate) fn interactive_marker_control_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerControl> {
    interactive_marker_control_from_view(&msg.view())
}

pub(crate) fn interactive_marker_control_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerControl,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/InteractiveMarkerControl")?;
    interactive_marker_control_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsInteractiveMarkerControlMapper;
impl TopicMapper for VisualizationMsgsInteractiveMarkerControlMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/InteractiveMarkerControl"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interactive_marker_control_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::InteractiveMarkerControl as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode visualization_msgs/msg/InteractiveMarkerControl: {e}"
                ))
            })?;
        interactive_marker_control_bus_to_dyn(&bus)
    }
}
