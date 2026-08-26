//! Mapper for `foxglove_msgs/msg/VoxelGrid`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn voxel_grid_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::VoxelGrid> {
    Ok(crate::foxglove_msgs::msg::v1::VoxelGrid {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        row_count: read_u32(view, "row_count")?,
        column_count: read_u32(view, "column_count")?,
        cell_size: nested_view(view, "cell_size")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        slice_stride: read_u32(view, "slice_stride")?,
        row_stride: read_u32(view, "row_stride")?,
        cell_stride: read_u32(view, "cell_stride")?,
        fields: read_message_seq(
            view,
            "fields",
            super::packed_element_field::packed_element_field_from_view,
        )?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn voxel_grid_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::VoxelGrid,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_u32(view, "row_count", bus.row_count)?;
    write_u32(view, "column_count", bus.column_count)?;
    if let Some(v) = &bus.cell_size {
        with_nested_mut(view, "cell_size", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    write_u32(view, "slice_stride", bus.slice_stride)?;
    write_u32(view, "row_stride", bus.row_stride)?;
    write_u32(view, "cell_stride", bus.cell_stride)?;
    write_message_seq(
        view,
        "fields",
        &bus.fields,
        super::packed_element_field::packed_element_field_write,
    )?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn voxel_grid_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::VoxelGrid> {
    voxel_grid_from_view(&msg.view())
}

pub(crate) fn voxel_grid_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::VoxelGrid,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/VoxelGrid")?;
    voxel_grid_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsVoxelGridMapper;
impl TopicMapper for FoxgloveMsgsVoxelGridMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/VoxelGrid"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(voxel_grid_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::VoxelGrid as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/VoxelGrid: {e}")))?;
        voxel_grid_bus_to_dyn(&bus)
    }
}
