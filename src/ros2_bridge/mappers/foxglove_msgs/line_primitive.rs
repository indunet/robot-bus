//! Mapper for `foxglove_msgs/msg/LinePrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn line_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::LinePrimitive> {
    Ok(crate::foxglove_msgs::msg::v1::LinePrimitive {
        r#type: read_i32(view, "type")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        thickness: read_f64(view, "thickness")?,
        scale_invariant: read_bool(view, "scale_invariant")?,
        points: read_message_seq(view, "points", super::point3::point3_from_view)?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        colors: read_message_seq(view, "colors", super::color::color_from_view)?,
        indices: read_u32_seq(view, "indices")?,
    })
}

pub(crate) fn line_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::LinePrimitive,
) -> Result<()> {
    write_i32(view, "type", bus.r#type)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_f64(view, "thickness", bus.thickness)?;
    write_bool(view, "scale_invariant", bus.scale_invariant)?;
    write_message_seq(view, "points", &bus.points, super::point3::point3_write)?;
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    write_message_seq(view, "colors", &bus.colors, super::color::color_write)?;
    write_u32_seq(view, "indices", &bus.indices)?;
    Ok(())
}

pub(crate) fn line_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::LinePrimitive> {
    line_primitive_from_view(&msg.view())
}

pub(crate) fn line_primitive_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::LinePrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/LinePrimitive")?;
    line_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsLinePrimitiveMapper;
impl TopicMapper for FoxgloveMsgsLinePrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LinePrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(line_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::LinePrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode foxglove_msgs/msg/LinePrimitive: {e}"))
        })?;
        line_primitive_bus_to_dyn(&bus)
    }
}
