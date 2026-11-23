//! Typed mapper for `sensor_msgs/msg/CompressedImage`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn compressed_image_to_bus(
    msg: ros_env::sensor_msgs::msg::CompressedImage,
) -> crate::sensor_msgs::msg::v1::CompressedImage {
    crate::sensor_msgs::msg::v1::CompressedImage {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        format: crate::ros2_bridge::mappers::convert::from_ros_string(msg.format),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn compressed_image_to_ros(
    bus: crate::sensor_msgs::msg::v1::CompressedImage,
) -> ros_env::sensor_msgs::msg::CompressedImage {
    ros_env::sensor_msgs::msg::CompressedImage {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        format: crate::ros2_bridge::mappers::convert::to_ros_string(bus.format),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsCompressedImageMapper;

impl TypedTopicMapper for SensorMsgsCompressedImageMapper {
    type Ros = ros_env::sensor_msgs::msg::CompressedImage;
    type Bus = crate::sensor_msgs::msg::v1::CompressedImage;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(compressed_image_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(compressed_image_to_ros(msg))
    }
}
