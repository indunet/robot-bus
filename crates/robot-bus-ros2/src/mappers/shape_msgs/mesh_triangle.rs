//! Typed mapper for `shape_msgs/msg/MeshTriangle`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn mesh_triangle_to_bus(
    msg: ros_env::shape_msgs::msg::MeshTriangle,
) -> robot_bus::shape_msgs::msg::v1::MeshTriangle {
    robot_bus::shape_msgs::msg::v1::MeshTriangle {
        vertex_indices: crate::mappers::convert::u32_seq(msg.vertex_indices),
    }
}

pub(crate) fn mesh_triangle_to_ros(
    bus: robot_bus::shape_msgs::msg::v1::MeshTriangle,
) -> ros_env::shape_msgs::msg::MeshTriangle {
    ros_env::shape_msgs::msg::MeshTriangle {
        vertex_indices: crate::mappers::convert::FromU32Seq::from_u32_seq(
            bus.vertex_indices,
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeMsgsMeshTriangleMapper;

impl TypedTopicMapper for ShapeMsgsMeshTriangleMapper {
    type Ros = ros_env::shape_msgs::msg::MeshTriangle;
    type Bus = robot_bus::shape_msgs::msg::v1::MeshTriangle;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(mesh_triangle_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(mesh_triangle_to_ros(msg))
    }
}
