//! Typed mapper for `nav_msgs/msg/MapMetaData`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn map_meta_data_to_bus(msg: ros_env::nav_msgs::msg::MapMetaData) -> crate::nav_msgs::msg::v1::MapMetaData {
    crate::nav_msgs::msg::v1::MapMetaData {
        map_load_time: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.map_load_time)),
        resolution: msg.resolution,
        width: msg.width.into(),
        height: msg.height.into(),
        origin: Some(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus(msg.origin)),
    }
}

pub(crate) fn map_meta_data_to_ros(bus: crate::nav_msgs::msg::v1::MapMetaData) -> ros_env::nav_msgs::msg::MapMetaData {
    ros_env::nav_msgs::msg::MapMetaData {
        map_load_time: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.map_load_time.unwrap_or_default()),
        resolution: bus.resolution,
        width: bus.width as _,
        height: bus.height as _,
        origin: crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros(bus.origin.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsMapMetaDataMapper;

impl TypedTopicMapper for NavMsgsMapMetaDataMapper {
    type Ros = ros_env::nav_msgs::msg::MapMetaData;
    type Bus = crate::nav_msgs::msg::v1::MapMetaData;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(map_meta_data_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(map_meta_data_to_ros(msg))
    }
}
