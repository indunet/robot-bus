//! Mapper for `foxglove_msgs/msg/TextPrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn text_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::TextPrimitive> {
    Ok(crate::foxglove_msgs::msg::v1::TextPrimitive {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        billboard: read_bool(view, "billboard")?,
        font_size: read_f64(view, "font_size")?,
        scale_invariant: read_bool(view, "scale_invariant")?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        text: read_string(view, "text")?,
    })
}

pub(crate) fn text_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::TextPrimitive,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_bool(view, "billboard", bus.billboard)?;
    write_f64(view, "font_size", bus.font_size)?;
    write_bool(view, "scale_invariant", bus.scale_invariant)?;
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    write_string(view, "text", &bus.text)?;
    Ok(())
}

pub(crate) fn text_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::TextPrimitive> {
    text_primitive_from_view(&msg.view())
}

pub(crate) fn text_primitive_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::TextPrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/TextPrimitive")?;
    text_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsTextPrimitiveMapper;
impl TopicMapper for FoxgloveMsgsTextPrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/TextPrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(text_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::TextPrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode foxglove_msgs/msg/TextPrimitive: {e}"))
        })?;
        text_primitive_bus_to_dyn(&bus)
    }
}
