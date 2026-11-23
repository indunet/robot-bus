//! Typed mapper for `sensor_msgs/msg/CompressedImage`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn compressed_image_to_bus(
    msg: ros_env::sensor_msgs::msg::CompressedImage,
) -> robot_bus::sensor_msgs::msg::v1::CompressedImage {
    robot_bus::sensor_msgs::msg::v1::CompressedImage {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        format: crate::mappers::convert::from_ros_string(msg.format),
        data: crate::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn compressed_image_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::CompressedImage,
) -> ros_env::sensor_msgs::msg::CompressedImage {
    ros_env::sensor_msgs::msg::CompressedImage {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        format: crate::mappers::convert::to_ros_string(bus.format),
        data: crate::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsCompressedImageMapper;

impl TypedTopicMapper for SensorMsgsCompressedImageMapper {
    type Ros = ros_env::sensor_msgs::msg::CompressedImage;
    type Bus = robot_bus::sensor_msgs::msg::v1::CompressedImage;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(compressed_image_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(compressed_image_to_ros(msg))
    }
}
