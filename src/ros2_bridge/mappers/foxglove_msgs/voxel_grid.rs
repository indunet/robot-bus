//! Typed mapper for `foxglove_msgs/msg/VoxelGrid`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn voxel_grid_to_bus(msg: ros_env::foxglove_msgs::msg::VoxelGrid) -> crate::foxglove_msgs::msg::v1::VoxelGrid {
    crate::foxglove_msgs::msg::v1::VoxelGrid {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        row_count: msg.row_count,
        column_count: msg.column_count,
        cell_size: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus(msg.cell_size)),
        slice_stride: msg.slice_stride,
        row_stride: msg.row_stride,
        cell_stride: msg.cell_stride,
        fields: msg.fields.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::packed_element_field::packed_element_field_to_bus).collect(),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn voxel_grid_to_ros(bus: crate::foxglove_msgs::msg::v1::VoxelGrid) -> ros_env::foxglove_msgs::msg::VoxelGrid {
    ros_env::foxglove_msgs::msg::VoxelGrid {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        row_count: bus.row_count,
        column_count: bus.column_count,
        cell_size: crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros(bus.cell_size.unwrap_or_default()),
        slice_stride: bus.slice_stride,
        row_stride: bus.row_stride,
        cell_stride: bus.cell_stride,
        fields: bus.fields.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::packed_element_field::packed_element_field_to_ros).collect(),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsVoxelGridMapper;

impl TypedTopicMapper for FoxgloveMsgsVoxelGridMapper {
    type Ros = ros_env::foxglove_msgs::msg::VoxelGrid;
    type Bus = crate::foxglove_msgs::msg::v1::VoxelGrid;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(voxel_grid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(voxel_grid_to_ros(msg))
    }
}
