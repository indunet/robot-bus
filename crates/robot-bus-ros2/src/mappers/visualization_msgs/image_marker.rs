//! Typed mapper for `visualization_msgs/msg/ImageMarker`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn image_marker_to_bus(
    msg: ros_env::visualization_msgs::msg::ImageMarker,
) -> robot_bus::visualization_msgs::msg::v1::ImageMarker {
    robot_bus::visualization_msgs::msg::v1::ImageMarker {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        ns: crate::mappers::convert::from_ros_string(msg.ns),
        id: msg.id.into(),
        r#type: msg.type_.into(),
        action: msg.action.into(),
        position: Some(
            crate::mappers::geometry_msgs::point::point_to_bus(msg.position),
        ),
        scale: msg.scale,
        outline_color: Some(
            crate::mappers::std_msgs::color_rgba::color_rgba_to_bus(msg.outline_color),
        ),
        filled: msg.filled.into(),
        fill_color: Some(
            crate::mappers::std_msgs::color_rgba::color_rgba_to_bus(msg.fill_color),
        ),
        lifetime: Some(
            crate::mappers::builtin_interfaces::duration::duration_to_bus(
                msg.lifetime,
            ),
        ),
        points: msg
            .points
            .into_iter()
            .map(crate::mappers::geometry_msgs::point::point_to_bus)
            .collect(),
        outline_colors: msg
            .outline_colors
            .into_iter()
            .map(crate::mappers::std_msgs::color_rgba::color_rgba_to_bus)
            .collect(),
    }
}

pub(crate) fn image_marker_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::ImageMarker,
) -> ros_env::visualization_msgs::msg::ImageMarker {
    ros_env::visualization_msgs::msg::ImageMarker {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        ns: crate::mappers::convert::to_ros_string(bus.ns),
        id: bus.id as _,
        type_: bus.r#type as _,
        action: bus.action as _,
        position: crate::mappers::geometry_msgs::point::point_to_ros(
            bus.position.unwrap_or_default(),
        ),
        scale: bus.scale,
        outline_color: crate::mappers::std_msgs::color_rgba::color_rgba_to_ros(
            bus.outline_color.unwrap_or_default(),
        ),
        filled: bus.filled as _,
        fill_color: crate::mappers::std_msgs::color_rgba::color_rgba_to_ros(
            bus.fill_color.unwrap_or_default(),
        ),
        lifetime: crate::mappers::builtin_interfaces::duration::duration_to_ros(
            bus.lifetime.unwrap_or_default(),
        ),
        points: bus
            .points
            .into_iter()
            .map(crate::mappers::geometry_msgs::point::point_to_ros)
            .collect(),
        outline_colors: bus
            .outline_colors
            .into_iter()
            .map(crate::mappers::std_msgs::color_rgba::color_rgba_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsImageMarkerMapper;

impl TypedTopicMapper for VisualizationMsgsImageMarkerMapper {
    type Ros = ros_env::visualization_msgs::msg::ImageMarker;
    type Bus = robot_bus::visualization_msgs::msg::v1::ImageMarker;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(image_marker_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(image_marker_to_ros(msg))
    }
}
