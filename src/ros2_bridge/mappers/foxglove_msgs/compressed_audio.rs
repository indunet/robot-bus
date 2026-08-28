//! Typed mapper for `foxglove_msgs/msg/CompressedAudio`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn compressed_audio_to_bus(msg: ros_env::foxglove_msgs::msg::CompressedAudio) -> crate::foxglove_msgs::msg::v1::CompressedAudio {
    crate::foxglove_msgs::msg::v1::CompressedAudio {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
        format: crate::ros2_bridge::mappers::convert::from_ros_string(msg.format),
    }
}

pub(crate) fn compressed_audio_to_ros(bus: crate::foxglove_msgs::msg::v1::CompressedAudio) -> ros_env::foxglove_msgs::msg::CompressedAudio {
    ros_env::foxglove_msgs::msg::CompressedAudio {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
        format: crate::ros2_bridge::mappers::convert::to_ros_string(bus.format),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCompressedAudioMapper;

impl TypedTopicMapper for FoxgloveMsgsCompressedAudioMapper {
    type Ros = ros_env::foxglove_msgs::msg::CompressedAudio;
    type Bus = crate::foxglove_msgs::msg::v1::CompressedAudio;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedAudio"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(compressed_audio_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(compressed_audio_to_ros(msg))
    }
}
