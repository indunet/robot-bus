//! Typed mapper for `nav2_msgs/msg/CostmapMetaData`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn costmap_meta_data_to_bus(msg: ros_env::nav2_msgs::msg::CostmapMetaData) -> crate::nav2_msgs::msg::v1::CostmapMetaData {
    crate::nav2_msgs::msg::v1::CostmapMetaData {
        map_load_time: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.map_load_time)),
        update_time: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.update_time)),
        resolution: msg.resolution,
        size_x: msg.size_x.into(),
        size_y: msg.size_y.into(),
        origin: Some(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus(msg.origin)),
        layer: crate::ros2_bridge::mappers::convert::from_ros_string(msg.layer),
    }
}

pub(crate) fn costmap_meta_data_to_ros(bus: crate::nav2_msgs::msg::v1::CostmapMetaData) -> ros_env::nav2_msgs::msg::CostmapMetaData {
    ros_env::nav2_msgs::msg::CostmapMetaData {
        map_load_time: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.map_load_time.unwrap_or_default()),
        update_time: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.update_time.unwrap_or_default()),
        resolution: bus.resolution,
        size_x: bus.size_x as _,
        size_y: bus.size_y as _,
        origin: crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros(bus.origin.unwrap_or_default()),
        layer: crate::ros2_bridge::mappers::convert::to_ros_string(bus.layer),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsCostmapMetaDataMapper;

impl TypedTopicMapper for Nav2MsgsCostmapMetaDataMapper {
    type Ros = ros_env::nav2_msgs::msg::CostmapMetaData;
    type Bus = crate::nav2_msgs::msg::v1::CostmapMetaData;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(costmap_meta_data_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(costmap_meta_data_to_ros(msg))
    }
}
