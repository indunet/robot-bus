//! Mapper for `foxglove_msgs/msg/SceneEntity`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn scene_entity_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::SceneEntity> {
    Ok(crate::foxglove_msgs::msg::v1::SceneEntity {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        id: read_string(view, "id")?,
        lifetime: read_duration(view, "lifetime")?,
        frame_locked: read_bool(view, "frame_locked")?,
        metadata: read_message_seq(view, "metadata", super::key_value_pair::key_value_pair_from_view)?,
        arrows: read_message_seq(view, "arrows", super::arrow_primitive::arrow_primitive_from_view)?,
        cubes: read_message_seq(view, "cubes", super::cube_primitive::cube_primitive_from_view)?,
        spheres: read_message_seq(view, "spheres", super::sphere_primitive::sphere_primitive_from_view)?,
        cylinders: read_message_seq(view, "cylinders", super::cylinder_primitive::cylinder_primitive_from_view)?,
        lines: read_message_seq(view, "lines", super::line_primitive::line_primitive_from_view)?,
        triangles: read_message_seq(view, "triangles", super::triangle_list_primitive::triangle_list_primitive_from_view)?,
        texts: read_message_seq(view, "texts", super::text_primitive::text_primitive_from_view)?,
        models: read_message_seq(view, "models", super::model_primitive::model_primitive_from_view)?,
    })
}

pub(crate) fn scene_entity_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::SceneEntity,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_string(view, "id", &bus.id)?;
    if let Some(v) = &bus.lifetime {
        write_duration(view, "lifetime", v)?;
    }
    write_bool(view, "frame_locked", bus.frame_locked)?;
    write_message_seq(view, "metadata", &bus.metadata, super::key_value_pair::key_value_pair_write)?;
    write_message_seq(view, "arrows", &bus.arrows, super::arrow_primitive::arrow_primitive_write)?;
    write_message_seq(view, "cubes", &bus.cubes, super::cube_primitive::cube_primitive_write)?;
    write_message_seq(view, "spheres", &bus.spheres, super::sphere_primitive::sphere_primitive_write)?;
    write_message_seq(view, "cylinders", &bus.cylinders, super::cylinder_primitive::cylinder_primitive_write)?;
    write_message_seq(view, "lines", &bus.lines, super::line_primitive::line_primitive_write)?;
    write_message_seq(
        view,
        "triangles",
        &bus.triangles,
        super::triangle_list_primitive::triangle_list_primitive_write,
    )?;
    write_message_seq(view, "texts", &bus.texts, super::text_primitive::text_primitive_write)?;
    write_message_seq(view, "models", &bus.models, super::model_primitive::model_primitive_write)?;
    Ok(())
}

pub(crate) fn scene_entity_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::SceneEntity> {
    scene_entity_from_view(&msg.view())
}

pub(crate) fn scene_entity_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::SceneEntity,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/SceneEntity")?;
    scene_entity_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsSceneEntityMapper;
impl TopicMapper for FoxgloveMsgsSceneEntityMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SceneEntity"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(scene_entity_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::SceneEntity as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/SceneEntity: {e}"))
            })?;
        scene_entity_bus_to_dyn(&bus)
    }
}
