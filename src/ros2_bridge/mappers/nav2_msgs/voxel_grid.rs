//! Typed mapper for `nav2_msgs/msg/VoxelGrid`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn voxel_grid_to_bus(msg: ros_env::nav2_msgs::msg::VoxelGrid) -> crate::nav2_msgs::msg::v1::VoxelGrid {
    crate::nav2_msgs::msg::v1::VoxelGrid {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        data: crate::ros2_bridge::mappers::convert::u32_seq(msg.data),
        origin: Some(crate::ros2_bridge::mappers::geometry_msgs::point32::point32_to_bus(msg.origin)),
        resolutions: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.resolutions)),
        size_x: msg.size_x.into(),
        size_y: msg.size_y.into(),
        size_z: msg.size_z.into(),
    }
}

pub(crate) fn voxel_grid_to_ros(bus: crate::nav2_msgs::msg::v1::VoxelGrid) -> ros_env::nav2_msgs::msg::VoxelGrid {
    ros_env::nav2_msgs::msg::VoxelGrid {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        data: crate::ros2_bridge::mappers::convert::FromU32Seq::from_u32_seq(bus.data),
        origin: crate::ros2_bridge::mappers::geometry_msgs::point32::point32_to_ros(bus.origin.unwrap_or_default()),
        resolutions: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.resolutions.unwrap_or_default()),
        size_x: bus.size_x as _,
        size_y: bus.size_y as _,
        size_z: bus.size_z as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsVoxelGridMapper;

impl TypedTopicMapper for Nav2MsgsVoxelGridMapper {
    type Ros = ros_env::nav2_msgs::msg::VoxelGrid;
    type Bus = crate::nav2_msgs::msg::v1::VoxelGrid;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(voxel_grid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(voxel_grid_to_ros(msg))
    }
}
