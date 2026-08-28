//! Typed mapper for `foxglove_msgs/msg/RawImage`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn raw_image_to_bus(msg: ros_env::foxglove_msgs::msg::RawImage) -> crate::foxglove_msgs::msg::v1::RawImage {
    crate::foxglove_msgs::msg::v1::RawImage {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        width: msg.width,
        height: msg.height,
        encoding: crate::ros2_bridge::mappers::convert::from_ros_string(msg.encoding),
        step: msg.step,
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn raw_image_to_ros(bus: crate::foxglove_msgs::msg::v1::RawImage) -> ros_env::foxglove_msgs::msg::RawImage {
    ros_env::foxglove_msgs::msg::RawImage {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        width: bus.width,
        height: bus.height,
        encoding: crate::ros2_bridge::mappers::convert::to_ros_string(bus.encoding),
        step: bus.step,
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsRawImageMapper;

impl TypedTopicMapper for FoxgloveMsgsRawImageMapper {
    type Ros = ros_env::foxglove_msgs::msg::RawImage;
    type Bus = crate::foxglove_msgs::msg::v1::RawImage;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/RawImage"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(raw_image_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(raw_image_to_ros(msg))
    }
}
