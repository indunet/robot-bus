//! Mapper for `foxglove_msgs/msg/PointsAnnotation`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn points_annotation_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::PointsAnnotation> {
    Ok(crate::foxglove_msgs::msg::v1::PointsAnnotation {
        timestamp: read_timestamp(view, "timestamp")?,
        r#type: read_i32(view, "type")?,
        points: read_message_seq(view, "points", super::point2::point2_from_view)?,
        outline_color: nested_view(view, "outline_color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        outline_colors: read_message_seq(view, "outline_colors", super::color::color_from_view)?,
        fill_color: nested_view(view, "fill_color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        thickness: read_f64(view, "thickness")?,
        metadata: read_message_seq(
            view,
            "metadata",
            super::key_value_pair::key_value_pair_from_view,
        )?,
    })
}

pub(crate) fn points_annotation_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::PointsAnnotation,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_i32(view, "type", bus.r#type)?;
    write_message_seq(view, "points", &bus.points, super::point2::point2_write)?;
    if let Some(v) = &bus.outline_color {
        with_nested_mut(view, "outline_color", |nested| {
            super::color::color_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "outline_colors",
        &bus.outline_colors,
        super::color::color_write,
    )?;
    if let Some(v) = &bus.fill_color {
        with_nested_mut(view, "fill_color", |nested| {
            super::color::color_write(nested, v)
        })?;
    }
    write_f64(view, "thickness", bus.thickness)?;
    write_message_seq(
        view,
        "metadata",
        &bus.metadata,
        super::key_value_pair::key_value_pair_write,
    )?;
    Ok(())
}

pub(crate) fn points_annotation_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::PointsAnnotation> {
    points_annotation_from_view(&msg.view())
}

pub(crate) fn points_annotation_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::PointsAnnotation,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/PointsAnnotation")?;
    points_annotation_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPointsAnnotationMapper;
impl TopicMapper for FoxgloveMsgsPointsAnnotationMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PointsAnnotation"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(points_annotation_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::PointsAnnotation as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode foxglove_msgs/msg/PointsAnnotation: {e}"))
                })?;
        points_annotation_bus_to_dyn(&bus)
    }
}
