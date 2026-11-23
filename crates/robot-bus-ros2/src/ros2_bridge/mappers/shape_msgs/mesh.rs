//! Typed mapper for `shape_msgs/msg/Mesh`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn mesh_to_bus(msg: ros_env::shape_msgs::msg::Mesh) -> crate::shape_msgs::msg::v1::Mesh {
    crate::shape_msgs::msg::v1::Mesh {
        triangles: msg
            .triangles
            .into_iter()
            .map(crate::ros2_bridge::mappers::shape_msgs::mesh_triangle::mesh_triangle_to_bus)
            .collect(),
        vertices: msg
            .vertices
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus)
            .collect(),
    }
}

pub(crate) fn mesh_to_ros(bus: crate::shape_msgs::msg::v1::Mesh) -> ros_env::shape_msgs::msg::Mesh {
    ros_env::shape_msgs::msg::Mesh {
        triangles: bus
            .triangles
            .into_iter()
            .map(crate::ros2_bridge::mappers::shape_msgs::mesh_triangle::mesh_triangle_to_ros)
            .collect(),
        vertices: bus
            .vertices
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeMsgsMeshMapper;

impl TypedTopicMapper for ShapeMsgsMeshMapper {
    type Ros = ros_env::shape_msgs::msg::Mesh;
    type Bus = crate::shape_msgs::msg::v1::Mesh;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(mesh_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(mesh_to_ros(msg))
    }
}
