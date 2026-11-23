//! Typed mapper for `sensor_msgs/msg/PointCloud`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn point_cloud_to_bus(
    msg: ros_env::sensor_msgs::msg::PointCloud,
) -> robot_bus::sensor_msgs::msg::v1::PointCloud {
    robot_bus::sensor_msgs::msg::v1::PointCloud {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        points: msg
            .points
            .into_iter()
            .map(crate::mappers::geometry_msgs::point32::point32_to_bus)
            .collect(),
        channels: msg
            .channels
            .into_iter()
            .map(crate::mappers::sensor_msgs::channel_float32::channel_float32_to_bus)
            .collect(),
    }
}

pub(crate) fn point_cloud_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::PointCloud,
) -> ros_env::sensor_msgs::msg::PointCloud {
    ros_env::sensor_msgs::msg::PointCloud {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        points: bus
            .points
            .into_iter()
            .map(crate::mappers::geometry_msgs::point32::point32_to_ros)
            .collect(),
        channels: bus
            .channels
            .into_iter()
            .map(crate::mappers::sensor_msgs::channel_float32::channel_float32_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsPointCloudMapper;

impl TypedTopicMapper for SensorMsgsPointCloudMapper {
    type Ros = ros_env::sensor_msgs::msg::PointCloud;
    type Bus = robot_bus::sensor_msgs::msg::v1::PointCloud;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(point_cloud_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(point_cloud_to_ros(msg))
    }
}
