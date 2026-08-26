//! Mapper for `foxglove_msgs/msg/CompressedAudio`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn compressed_audio_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedAudio> {
    Ok(crate::foxglove_msgs::msg::v1::CompressedAudio {
        timestamp: read_timestamp(view, "timestamp")?,
        data: read_byte_seq(view, "data")?,
        format: read_string(view, "format")?,
    })
}

pub(crate) fn compressed_audio_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CompressedAudio,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_byte_seq(view, "data", &bus.data)?;
    write_string(view, "format", &bus.format)?;
    Ok(())
}

pub(crate) fn compressed_audio_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedAudio> {
    compressed_audio_from_view(&msg.view())
}

pub(crate) fn compressed_audio_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CompressedAudio,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CompressedAudio")?;
    compressed_audio_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCompressedAudioMapper;
impl TopicMapper for FoxgloveMsgsCompressedAudioMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedAudio"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(compressed_audio_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::CompressedAudio as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/CompressedAudio: {e}"))
            })?;
        compressed_audio_bus_to_dyn(&bus)
    }
}
