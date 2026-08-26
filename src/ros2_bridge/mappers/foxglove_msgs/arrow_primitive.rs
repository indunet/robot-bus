//! Mapper for `foxglove_msgs/msg/ArrowPrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn arrow_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::ArrowPrimitive> {
    Ok(crate::foxglove_msgs::msg::v1::ArrowPrimitive {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        shaft_length: read_f64(view, "shaft_length")?,
        shaft_diameter: read_f64(view, "shaft_diameter")?,
        head_length: read_f64(view, "head_length")?,
        head_diameter: read_f64(view, "head_diameter")?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
    })
}

pub(crate) fn arrow_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::ArrowPrimitive,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_f64(view, "shaft_length", bus.shaft_length)?;
    write_f64(view, "shaft_diameter", bus.shaft_diameter)?;
    write_f64(view, "head_length", bus.head_length)?;
    write_f64(view, "head_diameter", bus.head_diameter)?;
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn arrow_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::ArrowPrimitive> {
    arrow_primitive_from_view(&msg.view())
}

pub(crate) fn arrow_primitive_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::ArrowPrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/ArrowPrimitive")?;
    arrow_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsArrowPrimitiveMapper;
impl TopicMapper for FoxgloveMsgsArrowPrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/ArrowPrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(arrow_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::ArrowPrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/ArrowPrimitive: {e}"))
            })?;
        arrow_primitive_bus_to_dyn(&bus)
    }
}
