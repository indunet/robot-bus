//! Mapper for `foxglove_msgs/msg/Point3InFrame`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point3_in_frame_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Point3InFrame> {
    Ok(crate::foxglove_msgs::msg::v1::Point3InFrame {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        point: nested_view(view, "point")?
            .as_ref()
            .map(super::point3::point3_from_view)
            .transpose()?,
    })
}

pub(crate) fn point3_in_frame_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Point3InFrame,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    if let Some(v) = &bus.point {
        with_nested_mut(view, "point", |nested| {
            super::point3::point3_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn point3_in_frame_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Point3InFrame> {
    point3_in_frame_from_view(&msg.view())
}

pub(crate) fn point3_in_frame_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Point3InFrame,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Point3InFrame")?;
    point3_in_frame_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPoint3InFrameMapper;
impl TopicMapper for FoxgloveMsgsPoint3InFrameMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Point3InFrame"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point3_in_frame_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Point3InFrame as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode foxglove_msgs/msg/Point3InFrame: {e}"))
        })?;
        point3_in_frame_bus_to_dyn(&bus)
    }
}
