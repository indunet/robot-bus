//! Typed mapper for `foxglove_msgs/msg/TextAnnotation`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn text_annotation_to_bus(msg: ros_env::foxglove_msgs::msg::TextAnnotation) -> crate::foxglove_msgs::msg::v1::TextAnnotation {
    crate::foxglove_msgs::msg::v1::TextAnnotation {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        position: Some(crate::ros2_bridge::mappers::foxglove_msgs::point2::point2_to_bus(msg.position)),
        text: crate::ros2_bridge::mappers::convert::from_ros_string(msg.text),
        font_size: msg.font_size,
        text_color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.text_color)),
        background_color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.background_color)),
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn text_annotation_to_ros(bus: crate::foxglove_msgs::msg::v1::TextAnnotation) -> ros_env::foxglove_msgs::msg::TextAnnotation {
    ros_env::foxglove_msgs::msg::TextAnnotation {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        position: crate::ros2_bridge::mappers::foxglove_msgs::point2::point2_to_ros(bus.position.unwrap_or_default()),
        text: crate::ros2_bridge::mappers::convert::to_ros_string(bus.text),
        font_size: bus.font_size,
        text_color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.text_color.unwrap_or_default()),
        background_color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.background_color.unwrap_or_default()),
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsTextAnnotationMapper;

impl TypedTopicMapper for FoxgloveMsgsTextAnnotationMapper {
    type Ros = ros_env::foxglove_msgs::msg::TextAnnotation;
    type Bus = crate::foxglove_msgs::msg::v1::TextAnnotation;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/TextAnnotation"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(text_annotation_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(text_annotation_to_ros(msg))
    }
}
