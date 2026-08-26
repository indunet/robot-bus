//! Mapper for `foxglove_msgs/msg/CubePrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn cube_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CubePrimitive> {
    Ok(crate::foxglove_msgs::msg::v1::CubePrimitive {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        size: nested_view(view, "size")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
    })
}

pub(crate) fn cube_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CubePrimitive,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    if let Some(v) = &bus.size {
        with_nested_mut(view, "size", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn cube_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CubePrimitive> {
    cube_primitive_from_view(&msg.view())
}

pub(crate) fn cube_primitive_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CubePrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CubePrimitive")?;
    cube_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCubePrimitiveMapper;
impl TopicMapper for FoxgloveMsgsCubePrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CubePrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(cube_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::CubePrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode foxglove_msgs/msg/CubePrimitive: {e}"))
        })?;
        cube_primitive_bus_to_dyn(&bus)
    }
}
