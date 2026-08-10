//! Mapper for `geometry_msgs/msg/Polygon`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn polygon_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Polygon> {
    Ok(crate::geometry_msgs::msg::v1::Polygon {
        points: read_message_seq(view, "points", super::point32::point32_from_view)?,
    })
}

pub(crate) fn polygon_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Polygon,
) -> Result<()> {
    write_message_seq(view, "points", &bus.points, super::point32::point32_write)?;
    Ok(())
}

pub(crate) fn polygon_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Polygon> {
    polygon_from_view(&msg.view())
}

pub(crate) fn polygon_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Polygon,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Polygon")?;
    polygon_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPolygonMapper;
impl TopicMapper for GeometryMsgsPolygonMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Polygon"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(polygon_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Polygon as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Polygon: {e}")))?;
        polygon_bus_to_dyn(&bus)
    }
}
