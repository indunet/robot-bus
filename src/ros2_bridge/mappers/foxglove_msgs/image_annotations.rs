//! Typed mapper for `foxglove_msgs/msg/ImageAnnotations`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn image_annotations_to_bus(msg: ros_env::foxglove_msgs::msg::ImageAnnotations) -> crate::foxglove_msgs::msg::v1::ImageAnnotations {
    crate::foxglove_msgs::msg::v1::ImageAnnotations {
        timestamp: msg.timestamp.map(crate::ros2_bridge::mappers::convert::time_to_timestamp),
        circles: msg.circles.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::circle_annotation::circle_annotation_to_bus).collect(),
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::points_annotation::points_annotation_to_bus).collect(),
        texts: msg.texts.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::text_annotation::text_annotation_to_bus).collect(),
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn image_annotations_to_ros(bus: crate::foxglove_msgs::msg::v1::ImageAnnotations) -> ros_env::foxglove_msgs::msg::ImageAnnotations {
    ros_env::foxglove_msgs::msg::ImageAnnotations {
        timestamp: bus.timestamp.map(crate::ros2_bridge::mappers::convert::timestamp_to_time),
        circles: bus.circles.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::circle_annotation::circle_annotation_to_ros).collect(),
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::points_annotation::points_annotation_to_ros).collect(),
        texts: bus.texts.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::text_annotation::text_annotation_to_ros).collect(),
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsImageAnnotationsMapper;

impl TypedTopicMapper for FoxgloveMsgsImageAnnotationsMapper {
    type Ros = ros_env::foxglove_msgs::msg::ImageAnnotations;
    type Bus = crate::foxglove_msgs::msg::v1::ImageAnnotations;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(image_annotations_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(image_annotations_to_ros(msg))
    }
}
