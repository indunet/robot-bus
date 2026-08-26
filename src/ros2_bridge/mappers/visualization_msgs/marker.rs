//! Mapper for `visualization_msgs/msg/Marker`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn marker_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::Marker> {
    Ok(crate::visualization_msgs::msg::v1::Marker {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        ns: read_string(view, "ns")?,
        id: read_i32(view, "id")?,
        r#type: read_i32(view, "type")?,
        action: read_i32(view, "action")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
        scale: nested_view(view, "scale")?
            .as_ref()
            .map(super::super::geometry_msgs::vector3::vector3_from_view)
            .transpose()?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::super::std_msgs::color_rgba::color_rgba_from_view)
            .transpose()?,
        lifetime: nested_view(view, "lifetime")?
            .as_ref()
            .map(super::super::builtin_interfaces::duration::duration_from_view)
            .transpose()?,
        frame_locked: read_bool(view, "frame_locked")?,
        points: read_message_seq(
            view,
            "points",
            super::super::geometry_msgs::point::point_from_view,
        )?,
        colors: read_message_seq(
            view,
            "colors",
            super::super::std_msgs::color_rgba::color_rgba_from_view,
        )?,
        texture_resource: read_string(view, "texture_resource")?,
        texture: nested_view(view, "texture")?
            .as_ref()
            .map(super::super::sensor_msgs::compressed_image::compressed_image_from_view)
            .transpose()?,
        uv_coordinates: read_message_seq(
            view,
            "uv_coordinates",
            super::uv_coordinate::uv_coordinate_from_view,
        )?,
        text: read_string(view, "text")?,
        mesh_resource: read_string(view, "mesh_resource")?,
        mesh_file: nested_view(view, "mesh_file")?
            .as_ref()
            .map(super::mesh_file::mesh_file_from_view)
            .transpose()?,
        mesh_use_embedded_materials: read_bool(view, "mesh_use_embedded_materials")?,
    })
}

pub(crate) fn marker_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::Marker,
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
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.scale {
        with_nested_mut(view, "scale", |nested| {
            super::super::geometry_msgs::vector3::vector3_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| {
            super::super::std_msgs::color_rgba::color_rgba_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.lifetime {
        with_nested_mut(view, "lifetime", |nested| {
            super::super::builtin_interfaces::duration::duration_write(nested, v)
        })?;
    }
    write_bool(view, "frame_locked", bus.frame_locked)?;
    write_message_seq(
        view,
        "points",
        &bus.points,
        super::super::geometry_msgs::point::point_write,
    )?;
    write_message_seq(
        view,
        "colors",
        &bus.colors,
        super::super::std_msgs::color_rgba::color_rgba_write,
    )?;
    write_string(view, "texture_resource", &bus.texture_resource)?;
    if let Some(v) = &bus.texture {
        with_nested_mut(view, "texture", |nested| {
            super::super::sensor_msgs::compressed_image::compressed_image_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "uv_coordinates",
        &bus.uv_coordinates,
        super::uv_coordinate::uv_coordinate_write,
    )?;
    write_string(view, "text", &bus.text)?;
    write_string(view, "mesh_resource", &bus.mesh_resource)?;
    if let Some(v) = &bus.mesh_file {
        with_nested_mut(view, "mesh_file", |nested| {
            super::mesh_file::mesh_file_write(nested, v)
        })?;
    }
    write_bool(
        view,
        "mesh_use_embedded_materials",
        bus.mesh_use_embedded_materials,
    )?;
    Ok(())
}

pub(crate) fn marker_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::Marker> {
    marker_from_view(&msg.view())
}

pub(crate) fn marker_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::Marker,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/Marker")?;
    marker_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsMarkerMapper;
impl TopicMapper for VisualizationMsgsMarkerMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/Marker"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(marker_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::visualization_msgs::msg::v1::Marker as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode visualization_msgs/msg/Marker: {e}"))
            })?;
        marker_bus_to_dyn(&bus)
    }
}
