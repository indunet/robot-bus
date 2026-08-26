//! Mapper for `foxglove_msgs/msg/RawAudio`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn raw_audio_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::RawAudio> {
    Ok(crate::foxglove_msgs::msg::v1::RawAudio {
        timestamp: read_timestamp(view, "timestamp")?,
        data: read_byte_seq(view, "data")?,
        format: read_string(view, "format")?,
        sample_rate: read_u32(view, "sample_rate")?,
        number_of_channels: read_u32(view, "number_of_channels")?,
    })
}

pub(crate) fn raw_audio_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::RawAudio,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_byte_seq(view, "data", &bus.data)?;
    write_string(view, "format", &bus.format)?;
    write_u32(view, "sample_rate", bus.sample_rate)?;
    write_u32(view, "number_of_channels", bus.number_of_channels)?;
    Ok(())
}

pub(crate) fn raw_audio_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::RawAudio> {
    raw_audio_from_view(&msg.view())
}

pub(crate) fn raw_audio_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::RawAudio,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/RawAudio")?;
    raw_audio_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsRawAudioMapper;
impl TopicMapper for FoxgloveMsgsRawAudioMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/RawAudio"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(raw_audio_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::RawAudio as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/RawAudio: {e}")))?;
        raw_audio_bus_to_dyn(&bus)
    }
}
