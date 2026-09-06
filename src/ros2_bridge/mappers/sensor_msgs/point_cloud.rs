//! Typed mapper for `sensor_msgs/msg/PointCloud`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_cloud_to_bus(
    msg: ros_env::sensor_msgs::msg::PointCloud,
) -> crate::sensor_msgs::msg::v1::PointCloud {
    crate::sensor_msgs::msg::v1::PointCloud {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        points: msg
            .points
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::point32::point32_to_bus)
            .collect(),
        channels: msg
            .channels
            .into_iter()
            .map(crate::ros2_bridge::mappers::sensor_msgs::channel_float32::channel_float32_to_bus)
            .collect(),
    }
}

pub(crate) fn point_cloud_to_ros(
    bus: crate::sensor_msgs::msg::v1::PointCloud,
) -> ros_env::sensor_msgs::msg::PointCloud {
    ros_env::sensor_msgs::msg::PointCloud {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        points: bus
            .points
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::point32::point32_to_ros)
            .collect(),
        channels: bus
            .channels
            .into_iter()
            .map(crate::ros2_bridge::mappers::sensor_msgs::channel_float32::channel_float32_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsPointCloudMapper;

impl TypedTopicMapper for SensorMsgsPointCloudMapper {
    type Ros = ros_env::sensor_msgs::msg::PointCloud;
    type Bus = crate::sensor_msgs::msg::v1::PointCloud;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_cloud_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_cloud_to_ros(msg))
    }
}
