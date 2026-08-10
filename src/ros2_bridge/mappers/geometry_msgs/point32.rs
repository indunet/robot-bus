//! Mapper for `geometry_msgs/msg/Point32`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn point32_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Point32> {
    Ok(crate::geometry_msgs::msg::v1::Point32 {
        x: read_f32(view, "x")?,
        y: read_f32(view, "y")?,
        z: read_f32(view, "z")?,
    })
}

pub(crate) fn point32_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Point32,
) -> Result<()> {
    write_f32(view, "x", bus.x)?;
    write_f32(view, "y", bus.y)?;
    write_f32(view, "z", bus.z)?;
    Ok(())
}

pub(crate) fn point32_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Point32> {
    point32_from_view(&msg.view())
}

pub(crate) fn point32_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Point32,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Point32")?;
    point32_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPoint32Mapper;
impl TopicMapper for GeometryMsgsPoint32Mapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Point32"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point32_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Point32 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Point32: {e}")))?;
        point32_bus_to_dyn(&bus)
    }
}
