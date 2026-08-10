//! Mapper for `shape_msgs/msg/MeshTriangle`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn mesh_triangle_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::shape_msgs::msg::v1::MeshTriangle> {
    Ok(crate::shape_msgs::msg::v1::MeshTriangle {
        vertex_indices: read_u32_seq(view, "vertex_indices")?,
    })
}

pub(crate) fn mesh_triangle_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::shape_msgs::msg::v1::MeshTriangle,
) -> Result<()> {
    write_u32_seq(view, "vertex_indices", &bus.vertex_indices)?;
    Ok(())
}

pub(crate) fn mesh_triangle_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::shape_msgs::msg::v1::MeshTriangle> {
    mesh_triangle_from_view(&msg.view())
}

pub(crate) fn mesh_triangle_bus_to_dyn(
    bus: &crate::shape_msgs::msg::v1::MeshTriangle,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("shape_msgs/msg/MeshTriangle")?;
    mesh_triangle_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ShapeMsgsMeshTriangleMapper;
impl TopicMapper for ShapeMsgsMeshTriangleMapper {
    fn type_name(&self) -> &'static str {
        "shape_msgs/msg/MeshTriangle"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(mesh_triangle_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::shape_msgs::msg::v1::MeshTriangle as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode shape_msgs/msg/MeshTriangle: {e}")))?;
        mesh_triangle_bus_to_dyn(&bus)
    }
}
