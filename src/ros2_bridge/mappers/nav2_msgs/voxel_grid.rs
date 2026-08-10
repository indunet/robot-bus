//! Mapper for `nav2_msgs/msg/VoxelGrid`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn voxel_grid_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::VoxelGrid> {
    Ok(crate::nav2_msgs::msg::v1::VoxelGrid {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        data: read_u32_seq(view, "data")?,
        origin: nested_view(view, "origin")?
            .as_ref()
            .map(super::super::geometry_msgs::point32::point32_from_view)
            .transpose()?,
        resolutions: nested_view(view, "resolutions")?
            .as_ref()
            .map(super::super::geometry_msgs::vector3::vector3_from_view)
            .transpose()?,
        size_x: read_u32(view, "size_x")?,
        size_y: read_u32(view, "size_y")?,
        size_z: read_u32(view, "size_z")?,
    })
}

pub(crate) fn voxel_grid_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::VoxelGrid,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32_seq(view, "data", &bus.data)?;
    if let Some(v) = &bus.origin {
        with_nested_mut(view, "origin", |nested| {
            super::super::geometry_msgs::point32::point32_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.resolutions {
        with_nested_mut(view, "resolutions", |nested| {
            super::super::geometry_msgs::vector3::vector3_write(nested, v)
        })?;
    }
    write_u32(view, "size_x", bus.size_x)?;
    write_u32(view, "size_y", bus.size_y)?;
    write_u32(view, "size_z", bus.size_z)?;
    Ok(())
}

pub(crate) fn voxel_grid_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::VoxelGrid> {
    voxel_grid_from_view(&msg.view())
}

pub(crate) fn voxel_grid_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::VoxelGrid,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/VoxelGrid")?;
    voxel_grid_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsVoxelGridMapper;
impl TopicMapper for Nav2MsgsVoxelGridMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/VoxelGrid"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(voxel_grid_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::VoxelGrid as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/VoxelGrid: {e}")))?;
        voxel_grid_bus_to_dyn(&bus)
    }
}
