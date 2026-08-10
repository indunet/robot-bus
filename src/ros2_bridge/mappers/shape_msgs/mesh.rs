//! Mapper for `shape_msgs/msg/Mesh`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn mesh_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::shape_msgs::msg::v1::Mesh> {
    Ok(crate::shape_msgs::msg::v1::Mesh {
        triangles: read_message_seq(view, "triangles", super::mesh_triangle::mesh_triangle_from_view)?,
        vertices: read_message_seq(view, "vertices", super::super::geometry_msgs::point::point_from_view)?,
    })
}

pub(crate) fn mesh_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::shape_msgs::msg::v1::Mesh,
) -> Result<()> {
    write_message_seq(view, "triangles", &bus.triangles, super::mesh_triangle::mesh_triangle_write)?;
    write_message_seq(
        view,
        "vertices",
        &bus.vertices,
        super::super::geometry_msgs::point::point_write,
    )?;
    Ok(())
}

pub(crate) fn mesh_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::shape_msgs::msg::v1::Mesh> {
    mesh_from_view(&msg.view())
}

pub(crate) fn mesh_bus_to_dyn(
    bus: &crate::shape_msgs::msg::v1::Mesh,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("shape_msgs/msg/Mesh")?;
    mesh_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ShapeMsgsMeshMapper;
impl TopicMapper for ShapeMsgsMeshMapper {
    fn type_name(&self) -> &'static str {
        "shape_msgs/msg/Mesh"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(mesh_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::shape_msgs::msg::v1::Mesh as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode shape_msgs/msg/Mesh: {e}")))?;
        mesh_bus_to_dyn(&bus)
    }
}
