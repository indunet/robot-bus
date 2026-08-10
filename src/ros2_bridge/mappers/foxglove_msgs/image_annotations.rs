//! Mapper for `foxglove_msgs/msg/ImageAnnotations`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn image_annotations_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::ImageAnnotations> {
    Ok(crate::foxglove_msgs::msg::v1::ImageAnnotations {
        timestamp: read_timestamp(view, "timestamp")?,
        circles: read_message_seq(view, "circles", super::circle_annotation::circle_annotation_from_view)?,
        points: read_message_seq(view, "points", super::points_annotation::points_annotation_from_view)?,
        texts: read_message_seq(view, "texts", super::text_annotation::text_annotation_from_view)?,
        metadata: read_message_seq(view, "metadata", super::key_value_pair::key_value_pair_from_view)?,
    })
}

pub(crate) fn image_annotations_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::ImageAnnotations,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_message_seq(view, "circles", &bus.circles, super::circle_annotation::circle_annotation_write)?;
    write_message_seq(view, "points", &bus.points, super::points_annotation::points_annotation_write)?;
    write_message_seq(view, "texts", &bus.texts, super::text_annotation::text_annotation_write)?;
    write_message_seq(view, "metadata", &bus.metadata, super::key_value_pair::key_value_pair_write)?;
    Ok(())
}

pub(crate) fn image_annotations_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::ImageAnnotations> {
    image_annotations_from_view(&msg.view())
}

pub(crate) fn image_annotations_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::ImageAnnotations,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/ImageAnnotations")?;
    image_annotations_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsImageAnnotationsMapper;
impl TopicMapper for FoxgloveMsgsImageAnnotationsMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/ImageAnnotations"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(image_annotations_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::ImageAnnotations as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode foxglove_msgs/msg/ImageAnnotations: {e}"))
                })?;
        image_annotations_bus_to_dyn(&bus)
    }
}
