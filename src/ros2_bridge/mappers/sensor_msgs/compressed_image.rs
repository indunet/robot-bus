//! Mapper for `sensor_msgs/msg/CompressedImage`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn compressed_image_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::CompressedImage> {
    Ok(crate::sensor_msgs::msg::v1::CompressedImage {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        format: read_string(view, "format")?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn compressed_image_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::CompressedImage,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string(view, "format", &bus.format)?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn compressed_image_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::CompressedImage> {
    compressed_image_from_view(&msg.view())
}

pub(crate) fn compressed_image_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::CompressedImage,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/CompressedImage")?;
    compressed_image_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsCompressedImageMapper;
impl TopicMapper for SensorMsgsCompressedImageMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/CompressedImage"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(compressed_image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::CompressedImage as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode sensor_msgs/msg/CompressedImage: {e}"))
        })?;
        compressed_image_bus_to_dyn(&bus)
    }
}
