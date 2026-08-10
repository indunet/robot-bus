//! Mapper for `foxglove_msgs/msg/ModelPrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn model_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::ModelPrimitive> {
    Ok(crate::foxglove_msgs::msg::v1::ModelPrimitive {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        scale: nested_view(view, "scale")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        override_color: read_bool(view, "override_color")?,
        url: read_string(view, "url")?,
        media_type: read_string(view, "media_type")?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn model_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::ModelPrimitive,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    if let Some(v) = &bus.scale {
        with_nested_mut(view, "scale", |nested| super::vector3::vector3_write(nested, v))?;
    }
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    write_bool(view, "override_color", bus.override_color)?;
    write_string(view, "url", &bus.url)?;
    write_string(view, "media_type", &bus.media_type)?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn model_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::ModelPrimitive> {
    model_primitive_from_view(&msg.view())
}

pub(crate) fn model_primitive_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::ModelPrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/ModelPrimitive")?;
    model_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsModelPrimitiveMapper;
impl TopicMapper for FoxgloveMsgsModelPrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/ModelPrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(model_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::ModelPrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/ModelPrimitive: {e}"))
            })?;
        model_primitive_bus_to_dyn(&bus)
    }
}
