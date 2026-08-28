//! Typed mapper for `foxglove_msgs/msg/CompressedVideo`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn compressed_video_to_bus(msg: ros_env::foxglove_msgs::msg::CompressedVideo) -> crate::foxglove_msgs::msg::v1::CompressedVideo {
    crate::foxglove_msgs::msg::v1::CompressedVideo {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
        format: crate::ros2_bridge::mappers::convert::from_ros_string(msg.format),
    }
}

pub(crate) fn compressed_video_to_ros(bus: crate::foxglove_msgs::msg::v1::CompressedVideo) -> ros_env::foxglove_msgs::msg::CompressedVideo {
    ros_env::foxglove_msgs::msg::CompressedVideo {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
        format: crate::ros2_bridge::mappers::convert::to_ros_string(bus.format),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCompressedVideoMapper;

impl TypedTopicMapper for FoxgloveMsgsCompressedVideoMapper {
    type Ros = ros_env::foxglove_msgs::msg::CompressedVideo;
    type Bus = crate::foxglove_msgs::msg::v1::CompressedVideo;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedVideo"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(compressed_video_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(compressed_video_to_ros(msg))
    }
}
