//! Typed mapper for `foxglove_msgs/msg/PointsAnnotation`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn points_annotation_to_bus(msg: ros_env::foxglove_msgs::msg::PointsAnnotation) -> crate::foxglove_msgs::msg::v1::PointsAnnotation {
    crate::foxglove_msgs::msg::v1::PointsAnnotation {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        r#type: msg.type_ as i32,
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::point2::point2_to_bus).collect(),
        outline_color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.outline_color)),
        outline_colors: msg.outline_colors.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus).collect(),
        fill_color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.fill_color)),
        thickness: msg.thickness,
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn points_annotation_to_ros(bus: crate::foxglove_msgs::msg::v1::PointsAnnotation) -> ros_env::foxglove_msgs::msg::PointsAnnotation {
    ros_env::foxglove_msgs::msg::PointsAnnotation {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        type_: bus.r#type as i32,
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::point2::point2_to_ros).collect(),
        outline_color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.outline_color.unwrap_or_default()),
        outline_colors: bus.outline_colors.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros).collect(),
        fill_color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.fill_color.unwrap_or_default()),
        thickness: bus.thickness,
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPointsAnnotationMapper;

impl TypedTopicMapper for FoxgloveMsgsPointsAnnotationMapper {
    type Ros = ros_env::foxglove_msgs::msg::PointsAnnotation;
    type Bus = crate::foxglove_msgs::msg::v1::PointsAnnotation;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(points_annotation_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(points_annotation_to_ros(msg))
    }
}
