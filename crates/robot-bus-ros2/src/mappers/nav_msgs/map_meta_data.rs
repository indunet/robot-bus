//! Typed mapper for `nav_msgs/msg/MapMetaData`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn map_meta_data_to_bus(
    msg: ros_env::nav_msgs::msg::MapMetaData,
) -> robot_bus::nav_msgs::msg::v1::MapMetaData {
    robot_bus::nav_msgs::msg::v1::MapMetaData {
        map_load_time: Some(
            crate::mappers::builtin_interfaces::time::time_to_bus(msg.map_load_time),
        ),
        resolution: msg.resolution,
        width: msg.width.into(),
        height: msg.height.into(),
        origin: Some(crate::mappers::geometry_msgs::pose::pose_to_bus(msg.origin)),
    }
}

pub(crate) fn map_meta_data_to_ros(
    bus: robot_bus::nav_msgs::msg::v1::MapMetaData,
) -> ros_env::nav_msgs::msg::MapMetaData {
    ros_env::nav_msgs::msg::MapMetaData {
        map_load_time: crate::mappers::builtin_interfaces::time::time_to_ros(
            bus.map_load_time.unwrap_or_default(),
        ),
        resolution: bus.resolution,
        width: bus.width as _,
        height: bus.height as _,
        origin: crate::mappers::geometry_msgs::pose::pose_to_ros(
            bus.origin.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsMapMetaDataMapper;

impl TypedTopicMapper for NavMsgsMapMetaDataMapper {
    type Ros = ros_env::nav_msgs::msg::MapMetaData;
    type Bus = robot_bus::nav_msgs::msg::v1::MapMetaData;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(map_meta_data_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(map_meta_data_to_ros(msg))
    }
}
