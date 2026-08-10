//! Mapper for `foxglove_msgs/msg/Point2`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn point2_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Point2> {
    Ok(crate::foxglove_msgs::msg::v1::Point2 {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
    })
}

pub(crate) fn point2_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Point2,
) -> Result<()> {
    write_f64(view, "x", bus.x)?;
    write_f64(view, "y", bus.y)?;
    Ok(())
}

pub(crate) fn point2_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Point2> {
    point2_from_view(&msg.view())
}

pub(crate) fn point2_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Point2,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Point2")?;
    point2_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPoint2Mapper;
impl TopicMapper for FoxgloveMsgsPoint2Mapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Point2"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point2_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Point2 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Point2: {e}")))?;
        point2_bus_to_dyn(&bus)
    }
}
