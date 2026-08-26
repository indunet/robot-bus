//! Mapper for `apriltag_msgs/msg/Point`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::apriltag_msgs::msg::v1::Point> {
    Ok(crate::apriltag_msgs::msg::v1::Point {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
    })
}

pub(crate) fn point_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::apriltag_msgs::msg::v1::Point,
) -> Result<()> {
    write_f64(view, "x", bus.x)?;
    write_f64(view, "y", bus.y)?;
    Ok(())
}

pub(crate) fn point_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::apriltag_msgs::msg::v1::Point> {
    point_from_view(&msg.view())
}

pub(crate) fn point_bus_to_dyn(
    bus: &crate::apriltag_msgs::msg::v1::Point,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("apriltag_msgs/msg/Point")?;
    point_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ApriltagMsgsPointMapper;
impl TopicMapper for ApriltagMsgsPointMapper {
    fn type_name(&self) -> &'static str {
        "apriltag_msgs/msg/Point"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::apriltag_msgs::msg::v1::Point as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode apriltag_msgs/msg/Point: {e}")))?;
        point_bus_to_dyn(&bus)
    }
}
