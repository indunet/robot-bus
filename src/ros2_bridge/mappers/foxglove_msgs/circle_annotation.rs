//! Mapper for `foxglove_msgs/msg/CircleAnnotation`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn circle_annotation_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CircleAnnotation> {
    Ok(crate::foxglove_msgs::msg::v1::CircleAnnotation {
        timestamp: read_timestamp(view, "timestamp")?,
        position: nested_view(view, "position")?
            .as_ref()
            .map(super::point2::point2_from_view)
            .transpose()?,
        diameter: read_f64(view, "diameter")?,
        thickness: read_f64(view, "thickness")?,
        fill_color: nested_view(view, "fill_color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        outline_color: nested_view(view, "outline_color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        metadata: read_message_seq(view, "metadata", super::key_value_pair::key_value_pair_from_view)?,
    })
}

pub(crate) fn circle_annotation_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CircleAnnotation,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    if let Some(v) = &bus.position {
        with_nested_mut(view, "position", |nested| super::point2::point2_write(nested, v))?;
    }
    write_f64(view, "diameter", bus.diameter)?;
    write_f64(view, "thickness", bus.thickness)?;
    if let Some(v) = &bus.fill_color {
        with_nested_mut(view, "fill_color", |nested| super::color::color_write(nested, v))?;
    }
    if let Some(v) = &bus.outline_color {
        with_nested_mut(view, "outline_color", |nested| super::color::color_write(nested, v))?;
    }
    write_message_seq(view, "metadata", &bus.metadata, super::key_value_pair::key_value_pair_write)?;
    Ok(())
}

pub(crate) fn circle_annotation_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CircleAnnotation> {
    circle_annotation_from_view(&msg.view())
}

pub(crate) fn circle_annotation_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CircleAnnotation,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CircleAnnotation")?;
    circle_annotation_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCircleAnnotationMapper;
impl TopicMapper for FoxgloveMsgsCircleAnnotationMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CircleAnnotation"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(circle_annotation_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::CircleAnnotation as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode foxglove_msgs/msg/CircleAnnotation: {e}"))
                })?;
        circle_annotation_bus_to_dyn(&bus)
    }
}
