//! Mapper for `foxglove_msgs/msg/TextAnnotation`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn text_annotation_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::TextAnnotation> {
    Ok(crate::foxglove_msgs::msg::v1::TextAnnotation {
        timestamp: read_timestamp(view, "timestamp")?,
        position: nested_view(view, "position")?
            .as_ref()
            .map(super::point2::point2_from_view)
            .transpose()?,
        text: read_string(view, "text")?,
        font_size: read_f64(view, "font_size")?,
        text_color: nested_view(view, "text_color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        background_color: nested_view(view, "background_color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        metadata: read_message_seq(
            view,
            "metadata",
            super::key_value_pair::key_value_pair_from_view,
        )?,
    })
}

pub(crate) fn text_annotation_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::TextAnnotation,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    if let Some(v) = &bus.position {
        with_nested_mut(view, "position", |nested| {
            super::point2::point2_write(nested, v)
        })?;
    }
    write_string(view, "text", &bus.text)?;
    write_f64(view, "font_size", bus.font_size)?;
    if let Some(v) = &bus.text_color {
        with_nested_mut(view, "text_color", |nested| {
            super::color::color_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.background_color {
        with_nested_mut(view, "background_color", |nested| {
            super::color::color_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "metadata",
        &bus.metadata,
        super::key_value_pair::key_value_pair_write,
    )?;
    Ok(())
}

pub(crate) fn text_annotation_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::TextAnnotation> {
    text_annotation_from_view(&msg.view())
}

pub(crate) fn text_annotation_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::TextAnnotation,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/TextAnnotation")?;
    text_annotation_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsTextAnnotationMapper;
impl TopicMapper for FoxgloveMsgsTextAnnotationMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/TextAnnotation"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(text_annotation_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::TextAnnotation as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/TextAnnotation: {e}"))
            })?;
        text_annotation_bus_to_dyn(&bus)
    }
}
