//! Mapper for `foxglove_msgs/msg/Vector2`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn vector2_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Vector2> {
    Ok(crate::foxglove_msgs::msg::v1::Vector2 {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
    })
}

pub(crate) fn vector2_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Vector2,
) -> Result<()> {
    write_f64(view, "x", bus.x)?;
    write_f64(view, "y", bus.y)?;
    Ok(())
}

pub(crate) fn vector2_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Vector2> {
    vector2_from_view(&msg.view())
}

pub(crate) fn vector2_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Vector2,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Vector2")?;
    vector2_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsVector2Mapper;
impl TopicMapper for FoxgloveMsgsVector2Mapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Vector2"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(vector2_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Vector2 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Vector2: {e}")))?;
        vector2_bus_to_dyn(&bus)
    }
}
