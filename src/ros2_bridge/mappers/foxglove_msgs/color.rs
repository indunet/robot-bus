//! Mapper for `foxglove_msgs/msg/Color`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn color_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Color> {
    Ok(crate::foxglove_msgs::msg::v1::Color {
        r: read_f64(view, "r")?,
        g: read_f64(view, "g")?,
        b: read_f64(view, "b")?,
        a: read_f64(view, "a")?,
    })
}

pub(crate) fn color_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Color,
) -> Result<()> {
    write_f64(view, "r", bus.r)?;
    write_f64(view, "g", bus.g)?;
    write_f64(view, "b", bus.b)?;
    write_f64(view, "a", bus.a)?;
    Ok(())
}

pub(crate) fn color_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Color> {
    color_from_view(&msg.view())
}

pub(crate) fn color_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Color,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Color")?;
    color_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsColorMapper;
impl TopicMapper for FoxgloveMsgsColorMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Color"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(color_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Color as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Color: {e}")))?;
        color_bus_to_dyn(&bus)
    }
}
