//! Typed mapper for `foxglove_msgs/msg/RawAudio`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn raw_audio_to_bus(msg: ros_env::foxglove_msgs::msg::RawAudio) -> crate::foxglove_msgs::msg::v1::RawAudio {
    crate::foxglove_msgs::msg::v1::RawAudio {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
        format: crate::ros2_bridge::mappers::convert::from_ros_string(msg.format),
        sample_rate: msg.sample_rate,
        number_of_channels: msg.number_of_channels,
    }
}

pub(crate) fn raw_audio_to_ros(bus: crate::foxglove_msgs::msg::v1::RawAudio) -> ros_env::foxglove_msgs::msg::RawAudio {
    ros_env::foxglove_msgs::msg::RawAudio {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
        format: crate::ros2_bridge::mappers::convert::to_ros_string(bus.format),
        sample_rate: bus.sample_rate,
        number_of_channels: bus.number_of_channels,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsRawAudioMapper;

impl TypedTopicMapper for FoxgloveMsgsRawAudioMapper {
    type Ros = ros_env::foxglove_msgs::msg::RawAudio;
    type Bus = crate::foxglove_msgs::msg::v1::RawAudio;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(raw_audio_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(raw_audio_to_ros(msg))
    }
}
