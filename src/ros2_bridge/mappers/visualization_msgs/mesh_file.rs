//! Mapper for `visualization_msgs/msg/MeshFile`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn mesh_file_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::MeshFile> {
    Ok(crate::visualization_msgs::msg::v1::MeshFile {
        filename: read_string(view, "filename")?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn mesh_file_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::MeshFile,
) -> Result<()> {
    write_string(view, "filename", &bus.filename)?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn mesh_file_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::MeshFile> {
    mesh_file_from_view(&msg.view())
}

pub(crate) fn mesh_file_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::MeshFile,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/MeshFile")?;
    mesh_file_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsMeshFileMapper;
impl TopicMapper for VisualizationMsgsMeshFileMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/MeshFile"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(mesh_file_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::visualization_msgs::msg::v1::MeshFile as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode visualization_msgs/msg/MeshFile: {e}"))
        })?;
        mesh_file_bus_to_dyn(&bus)
    }
}
