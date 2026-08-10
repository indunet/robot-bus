//! Mapper for `visualization_msgs/msg/ImageMarker`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn image_marker_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::ImageMarker> {
    Ok(crate::visualization_msgs::msg::v1::ImageMarker {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        ns: read_string(view, "ns")?,
        id: read_i32(view, "id")?,
        r#type: read_i32(view, "type")?,
        action: read_i32(view, "action")?,
        position: nested_view(view, "position")?
            .as_ref()
            .map(super::super::geometry_msgs::point::point_from_view)
            .transpose()?,
        scale: read_f32(view, "scale")?,
        outline_color: nested_view(view, "outline_color")?
            .as_ref()
            .map(super::super::std_msgs::color_rgba::color_rgba_from_view)
            .transpose()?,
        filled: read_u32(view, "filled")?,
        fill_color: nested_view(view, "fill_color")?
            .as_ref()
            .map(super::super::std_msgs::color_rgba::color_rgba_from_view)
            .transpose()?,
        lifetime: nested_view(view, "lifetime")?
            .as_ref()
            .map(super::super::builtin_interfaces::duration::duration_from_view)
            .transpose()?,
        points: read_message_seq(view, "points", super::super::geometry_msgs::point::point_from_view)?,
        outline_colors: read_message_seq(
            view,
            "outline_colors",
            super::super::std_msgs::color_rgba::color_rgba_from_view,
        )?,
    })
}

pub(crate) fn image_marker_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::ImageMarker,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string(view, "ns", &bus.ns)?;
    write_i32(view, "id", bus.id)?;
    write_i32(view, "type", bus.r#type)?;
    write_i32(view, "action", bus.action)?;
    if let Some(v) = &bus.position {
        with_nested_mut(view, "position", |nested| {
            super::super::geometry_msgs::point::point_write(nested, v)
        })?;
    }
    write_f32(view, "scale", bus.scale)?;
    if let Some(v) = &bus.outline_color {
        with_nested_mut(view, "outline_color", |nested| {
            super::super::std_msgs::color_rgba::color_rgba_write(nested, v)
        })?;
    }
    write_u32(view, "filled", bus.filled)?;
    if let Some(v) = &bus.fill_color {
        with_nested_mut(view, "fill_color", |nested| {
            super::super::std_msgs::color_rgba::color_rgba_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.lifetime {
        with_nested_mut(view, "lifetime", |nested| {
            super::super::builtin_interfaces::duration::duration_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "points",
        &bus.points,
        super::super::geometry_msgs::point::point_write,
    )?;
    write_message_seq(
        view,
        "outline_colors",
        &bus.outline_colors,
        super::super::std_msgs::color_rgba::color_rgba_write,
    )?;
    Ok(())
}

pub(crate) fn image_marker_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::ImageMarker> {
    image_marker_from_view(&msg.view())
}

pub(crate) fn image_marker_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::ImageMarker,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/ImageMarker")?;
    image_marker_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsImageMarkerMapper;
impl TopicMapper for VisualizationMsgsImageMarkerMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/ImageMarker"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(image_marker_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::ImageMarker as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode visualization_msgs/msg/ImageMarker: {e}"))
                })?;
        image_marker_bus_to_dyn(&bus)
    }
}
