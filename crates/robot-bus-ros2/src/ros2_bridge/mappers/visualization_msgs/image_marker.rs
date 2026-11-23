//! Typed mapper for `visualization_msgs/msg/ImageMarker`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn image_marker_to_bus(
    msg: ros_env::visualization_msgs::msg::ImageMarker,
) -> crate::visualization_msgs::msg::v1::ImageMarker {
    crate::visualization_msgs::msg::v1::ImageMarker {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        ns: crate::ros2_bridge::mappers::convert::from_ros_string(msg.ns),
        id: msg.id.into(),
        r#type: msg.type_.into(),
        action: msg.action.into(),
        position: Some(
            crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus(msg.position),
        ),
        scale: msg.scale,
        outline_color: Some(
            crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_bus(msg.outline_color),
        ),
        filled: msg.filled.into(),
        fill_color: Some(
            crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_bus(msg.fill_color),
        ),
        lifetime: Some(
            crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_bus(
                msg.lifetime,
            ),
        ),
        points: msg
            .points
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus)
            .collect(),
        outline_colors: msg
            .outline_colors
            .into_iter()
            .map(crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_bus)
            .collect(),
    }
}

pub(crate) fn image_marker_to_ros(
    bus: crate::visualization_msgs::msg::v1::ImageMarker,
) -> ros_env::visualization_msgs::msg::ImageMarker {
    ros_env::visualization_msgs::msg::ImageMarker {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        ns: crate::ros2_bridge::mappers::convert::to_ros_string(bus.ns),
        id: bus.id as _,
        type_: bus.r#type as _,
        action: bus.action as _,
        position: crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros(
            bus.position.unwrap_or_default(),
        ),
        scale: bus.scale,
        outline_color: crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_ros(
            bus.outline_color.unwrap_or_default(),
        ),
        filled: bus.filled as _,
        fill_color: crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_ros(
            bus.fill_color.unwrap_or_default(),
        ),
        lifetime: crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_ros(
            bus.lifetime.unwrap_or_default(),
        ),
        points: bus
            .points
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros)
            .collect(),
        outline_colors: bus
            .outline_colors
            .into_iter()
            .map(crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsImageMarkerMapper;

impl TypedTopicMapper for VisualizationMsgsImageMarkerMapper {
    type Ros = ros_env::visualization_msgs::msg::ImageMarker;
    type Bus = crate::visualization_msgs::msg::v1::ImageMarker;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(image_marker_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(image_marker_to_ros(msg))
    }
}
