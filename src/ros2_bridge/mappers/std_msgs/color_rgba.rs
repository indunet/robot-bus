//! Mapper for `std_msgs/msg/ColorRGBA`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn color_rgba_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::ColorRgba> {
    Ok(crate::std_msgs::msg::v1::ColorRgba {
        r: read_f32(view, "r")?,
        g: read_f32(view, "g")?,
        b: read_f32(view, "b")?,
        a: read_f32(view, "a")?,
    })
}

pub(crate) fn color_rgba_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::ColorRgba,
) -> Result<()> {
    write_f32(view, "r", bus.r)?;
    write_f32(view, "g", bus.g)?;
    write_f32(view, "b", bus.b)?;
    write_f32(view, "a", bus.a)?;
    Ok(())
}

pub(crate) fn color_rgba_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::ColorRgba> {
    color_rgba_from_view(&msg.view())
}

pub(crate) fn color_rgba_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::ColorRgba,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/ColorRGBA")?;
    color_rgba_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsColorRgbaMapper;
impl TopicMapper for StdMsgsColorRgbaMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/ColorRGBA"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(color_rgba_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::ColorRgba as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/ColorRGBA: {e}")))?;
        color_rgba_bus_to_dyn(&bus)
    }
}
