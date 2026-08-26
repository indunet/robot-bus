//! Mapper for `foxglove_msgs/msg/CylinderPrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn cylinder_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CylinderPrimitive> {
    Ok(crate::foxglove_msgs::msg::v1::CylinderPrimitive {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        size: nested_view(view, "size")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        bottom_scale: read_f64(view, "bottom_scale")?,
        top_scale: read_f64(view, "top_scale")?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
    })
}

pub(crate) fn cylinder_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CylinderPrimitive,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    if let Some(v) = &bus.size {
        with_nested_mut(view, "size", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    write_f64(view, "bottom_scale", bus.bottom_scale)?;
    write_f64(view, "top_scale", bus.top_scale)?;
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn cylinder_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CylinderPrimitive> {
    cylinder_primitive_from_view(&msg.view())
}

pub(crate) fn cylinder_primitive_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CylinderPrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CylinderPrimitive")?;
    cylinder_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCylinderPrimitiveMapper;
impl TopicMapper for FoxgloveMsgsCylinderPrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CylinderPrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(cylinder_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::CylinderPrimitive as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode foxglove_msgs/msg/CylinderPrimitive: {e}"))
                })?;
        cylinder_primitive_bus_to_dyn(&bus)
    }
}
