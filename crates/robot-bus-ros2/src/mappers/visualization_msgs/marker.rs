//! Typed mapper for `visualization_msgs/msg/Marker`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn marker_to_bus(
    msg: ros_env::visualization_msgs::msg::Marker,
) -> robot_bus::visualization_msgs::msg::v1::Marker {
    robot_bus::visualization_msgs::msg::v1::Marker {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        ns: crate::mappers::convert::from_ros_string(msg.ns),
        id: msg.id.into(),
        r#type: msg.type_.into(),
        action: msg.action.into(),
        pose: Some(crate::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        scale: Some(crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.scale)),
        color: Some(crate::mappers::std_msgs::color_rgba::color_rgba_to_bus(msg.color)),
        lifetime: Some(crate::mappers::builtin_interfaces::duration::duration_to_bus(msg.lifetime)),
        frame_locked: msg.frame_locked,
        points: msg.points.into_iter().map(crate::mappers::geometry_msgs::point::point_to_bus).collect(),
        colors: msg.colors.into_iter().map(crate::mappers::std_msgs::color_rgba::color_rgba_to_bus).collect(),
        texture_resource: crate::mappers::convert::from_ros_string(msg.texture_resource),
        texture: Some(crate::mappers::sensor_msgs::compressed_image::compressed_image_to_bus(msg.texture)),
        uv_coordinates: msg.uv_coordinates.into_iter().map(crate::mappers::visualization_msgs::uv_coordinate::uv_coordinate_to_bus).collect(),
        text: crate::mappers::convert::from_ros_string(msg.text),
        mesh_resource: crate::mappers::convert::from_ros_string(msg.mesh_resource),
        mesh_file: Some(crate::mappers::visualization_msgs::mesh_file::mesh_file_to_bus(msg.mesh_file)),
        mesh_use_embedded_materials: msg.mesh_use_embedded_materials,
    }
}

pub(crate) fn marker_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::Marker,
) -> ros_env::visualization_msgs::msg::Marker {
    ros_env::visualization_msgs::msg::Marker {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        ns: crate::mappers::convert::to_ros_string(bus.ns),
        id: bus.id as _,
        type_: bus.r#type as _,
        action: bus.action as _,
        pose: crate::mappers::geometry_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        scale: crate::mappers::geometry_msgs::vector3::vector3_to_ros(bus.scale.unwrap_or_default()),
        color: crate::mappers::std_msgs::color_rgba::color_rgba_to_ros(bus.color.unwrap_or_default()),
        lifetime: crate::mappers::builtin_interfaces::duration::duration_to_ros(bus.lifetime.unwrap_or_default()),
        frame_locked: bus.frame_locked,
        points: bus.points.into_iter().map(crate::mappers::geometry_msgs::point::point_to_ros).collect(),
        colors: bus.colors.into_iter().map(crate::mappers::std_msgs::color_rgba::color_rgba_to_ros).collect(),
        texture_resource: crate::mappers::convert::to_ros_string(bus.texture_resource),
        texture: crate::mappers::sensor_msgs::compressed_image::compressed_image_to_ros(bus.texture.unwrap_or_default()),
        uv_coordinates: bus.uv_coordinates.into_iter().map(crate::mappers::visualization_msgs::uv_coordinate::uv_coordinate_to_ros).collect(),
        text: crate::mappers::convert::to_ros_string(bus.text),
        mesh_resource: crate::mappers::convert::to_ros_string(bus.mesh_resource),
        mesh_file: crate::mappers::visualization_msgs::mesh_file::mesh_file_to_ros(bus.mesh_file.unwrap_or_default()),
        mesh_use_embedded_materials: bus.mesh_use_embedded_materials,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsMarkerMapper;

impl TypedTopicMapper for VisualizationMsgsMarkerMapper {
    type Ros = ros_env::visualization_msgs::msg::Marker;
    type Bus = robot_bus::visualization_msgs::msg::v1::Marker;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(marker_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(marker_to_ros(msg))
    }
}
