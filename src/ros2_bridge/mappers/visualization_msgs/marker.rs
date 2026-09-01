//! Typed mapper for `visualization_msgs/msg/Marker`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn marker_to_bus(msg: ros_env::visualization_msgs::msg::Marker) -> crate::visualization_msgs::msg::v1::Marker {
    crate::visualization_msgs::msg::v1::Marker {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        ns: crate::ros2_bridge::mappers::convert::from_ros_string(msg.ns),
        id: msg.id,
        r#type: msg.type_,
        action: msg.action,
        pose: Some(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        scale: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.scale)),
        color: Some(crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_bus(msg.color)),
        lifetime: Some(crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_bus(msg.lifetime)),
        frame_locked: msg.frame_locked,
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus).collect(),
        colors: msg.colors.into_iter().map(crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_bus).collect(),
        texture_resource: crate::ros2_bridge::mappers::convert::from_ros_string(msg.texture_resource),
        texture: Some(crate::ros2_bridge::mappers::sensor_msgs::compressed_image::compressed_image_to_bus(msg.texture)),
        uv_coordinates: msg.uv_coordinates.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::uv_coordinate::uv_coordinate_to_bus).collect(),
        text: crate::ros2_bridge::mappers::convert::from_ros_string(msg.text),
        mesh_resource: crate::ros2_bridge::mappers::convert::from_ros_string(msg.mesh_resource),
        mesh_file: Some(crate::ros2_bridge::mappers::visualization_msgs::mesh_file::mesh_file_to_bus(msg.mesh_file)),
        mesh_use_embedded_materials: msg.mesh_use_embedded_materials,
    }
}

pub(crate) fn marker_to_ros(bus: crate::visualization_msgs::msg::v1::Marker) -> ros_env::visualization_msgs::msg::Marker {
    ros_env::visualization_msgs::msg::Marker {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        ns: crate::ros2_bridge::mappers::convert::to_ros_string(bus.ns),
        id: bus.id,
        type_: bus.r#type,
        action: bus.action,
        pose: crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        scale: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.scale.unwrap_or_default()),
        color: crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_ros(bus.color.unwrap_or_default()),
        lifetime: crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_ros(bus.lifetime.unwrap_or_default()),
        frame_locked: bus.frame_locked,
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros).collect(),
        colors: bus.colors.into_iter().map(crate::ros2_bridge::mappers::std_msgs::color_rgba::color_rgba_to_ros).collect(),
        texture_resource: crate::ros2_bridge::mappers::convert::to_ros_string(bus.texture_resource),
        texture: crate::ros2_bridge::mappers::sensor_msgs::compressed_image::compressed_image_to_ros(bus.texture.unwrap_or_default()),
        uv_coordinates: bus.uv_coordinates.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::uv_coordinate::uv_coordinate_to_ros).collect(),
        text: crate::ros2_bridge::mappers::convert::to_ros_string(bus.text),
        mesh_resource: crate::ros2_bridge::mappers::convert::to_ros_string(bus.mesh_resource),
        mesh_file: crate::ros2_bridge::mappers::visualization_msgs::mesh_file::mesh_file_to_ros(bus.mesh_file.unwrap_or_default()),
        mesh_use_embedded_materials: bus.mesh_use_embedded_materials,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsMarkerMapper;

impl TypedTopicMapper for VisualizationMsgsMarkerMapper {
    type Ros = ros_env::visualization_msgs::msg::Marker;
    type Bus = crate::visualization_msgs::msg::v1::Marker;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(marker_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(marker_to_ros(msg))
    }
}
