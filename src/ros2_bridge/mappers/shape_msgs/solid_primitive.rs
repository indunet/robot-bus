//! Mapper for `shape_msgs/msg/SolidPrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn solid_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::shape_msgs::msg::v1::SolidPrimitive> {
    Ok(crate::shape_msgs::msg::v1::SolidPrimitive {
        r#type: read_u32(view, "type")?,
        dimensions: read_f64_seq(view, "dimensions")?,
        polygon: nested_view(view, "polygon")?
            .as_ref()
            .map(super::super::geometry_msgs::polygon::polygon_from_view)
            .transpose()?,
    })
}

pub(crate) fn solid_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::shape_msgs::msg::v1::SolidPrimitive,
) -> Result<()> {
    write_u32(view, "type", bus.r#type)?;
    write_f64_seq(view, "dimensions", &bus.dimensions)?;
    if let Some(v) = &bus.polygon {
        with_nested_mut(view, "polygon", |nested| {
            super::super::geometry_msgs::polygon::polygon_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn solid_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::shape_msgs::msg::v1::SolidPrimitive> {
    solid_primitive_from_view(&msg.view())
}

pub(crate) fn solid_primitive_bus_to_dyn(
    bus: &crate::shape_msgs::msg::v1::SolidPrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("shape_msgs/msg/SolidPrimitive")?;
    solid_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ShapeMsgsSolidPrimitiveMapper;
impl TopicMapper for ShapeMsgsSolidPrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "shape_msgs/msg/SolidPrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(solid_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::shape_msgs::msg::v1::SolidPrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode shape_msgs/msg/SolidPrimitive: {e}"))
            })?;
        solid_primitive_bus_to_dyn(&bus)
    }
}
