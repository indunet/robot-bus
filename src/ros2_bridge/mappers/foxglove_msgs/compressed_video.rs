//! Mapper for `foxglove_msgs/msg/CompressedVideo`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn compressed_video_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedVideo> {
    Ok(crate::foxglove_msgs::msg::v1::CompressedVideo {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        data: read_byte_seq(view, "data")?,
        format: read_string(view, "format")?,
    })
}

pub(crate) fn compressed_video_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CompressedVideo,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_byte_seq(view, "data", &bus.data)?;
    write_string(view, "format", &bus.format)?;
    Ok(())
}

pub(crate) fn compressed_video_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedVideo> {
    compressed_video_from_view(&msg.view())
}

pub(crate) fn compressed_video_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CompressedVideo,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CompressedVideo")?;
    compressed_video_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCompressedVideoMapper;
impl TopicMapper for FoxgloveMsgsCompressedVideoMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedVideo"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(compressed_video_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::CompressedVideo as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/CompressedVideo: {e}"))
            })?;
        compressed_video_bus_to_dyn(&bus)
    }
}
