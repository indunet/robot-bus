//! Mapper for `foxglove_msgs/msg/CompressedImage`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn compressed_image_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedImage> {
    Ok(crate::foxglove_msgs::msg::v1::CompressedImage {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        data: read_byte_seq(view, "data")?,
        format: read_string(view, "format")?,
    })
}

pub(crate) fn compressed_image_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CompressedImage,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_byte_seq(view, "data", &bus.data)?;
    write_string(view, "format", &bus.format)?;
    Ok(())
}

pub(crate) fn compressed_image_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedImage> {
    compressed_image_from_view(&msg.view())
}

pub(crate) fn compressed_image_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CompressedImage,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CompressedImage")?;
    compressed_image_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCompressedImageMapper;
impl TopicMapper for FoxgloveMsgsCompressedImageMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedImage"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(compressed_image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::CompressedImage as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/CompressedImage: {e}"))
            })?;
        compressed_image_bus_to_dyn(&bus)
    }
}
