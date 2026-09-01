//! Typed mapper for `foxglove_msgs/msg/CircleAnnotation`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn circle_annotation_to_bus(msg: ros_env::foxglove_msgs::msg::CircleAnnotation) -> crate::foxglove_msgs::msg::v1::CircleAnnotation {
    crate::foxglove_msgs::msg::v1::CircleAnnotation {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        position: Some(crate::ros2_bridge::mappers::foxglove_msgs::point2::point2_to_bus(msg.position)),
        diameter: msg.diameter,
        thickness: msg.thickness,
        fill_color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.fill_color)),
        outline_color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.outline_color)),
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn circle_annotation_to_ros(bus: crate::foxglove_msgs::msg::v1::CircleAnnotation) -> ros_env::foxglove_msgs::msg::CircleAnnotation {
    ros_env::foxglove_msgs::msg::CircleAnnotation {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        position: crate::ros2_bridge::mappers::foxglove_msgs::point2::point2_to_ros(bus.position.unwrap_or_default()),
        diameter: bus.diameter,
        thickness: bus.thickness,
        fill_color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.fill_color.unwrap_or_default()),
        outline_color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.outline_color.unwrap_or_default()),
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCircleAnnotationMapper;

impl TypedTopicMapper for FoxgloveMsgsCircleAnnotationMapper {
    type Ros = ros_env::foxglove_msgs::msg::CircleAnnotation;
    type Bus = crate::foxglove_msgs::msg::v1::CircleAnnotation;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(circle_annotation_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(circle_annotation_to_ros(msg))
    }
}
