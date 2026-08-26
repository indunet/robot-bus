//! Mapper for `foxglove_msgs/msg/RawImage`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn raw_image_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::RawImage> {
    Ok(crate::foxglove_msgs::msg::v1::RawImage {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        width: read_u32(view, "width")?,
        height: read_u32(view, "height")?,
        encoding: read_string(view, "encoding")?,
        step: read_u32(view, "step")?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn raw_image_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::RawImage,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_u32(view, "width", bus.width)?;
    write_u32(view, "height", bus.height)?;
    write_string(view, "encoding", &bus.encoding)?;
    write_u32(view, "step", bus.step)?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn raw_image_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::RawImage> {
    raw_image_from_view(&msg.view())
}

pub(crate) fn raw_image_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::RawImage,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/RawImage")?;
    raw_image_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsRawImageMapper;
impl TopicMapper for FoxgloveMsgsRawImageMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/RawImage"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(raw_image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::RawImage as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/RawImage: {e}")))?;
        raw_image_bus_to_dyn(&bus)
    }
}
