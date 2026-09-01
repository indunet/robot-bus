//! Typed mapper for `foxglove_msgs/msg/CompressedPointCloud`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn compressed_point_cloud_to_bus(msg: ros_env::foxglove_msgs::msg::CompressedPointCloud) -> crate::foxglove_msgs::msg::v1::CompressedPointCloud {
    crate::foxglove_msgs::msg::v1::CompressedPointCloud {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
        format: crate::ros2_bridge::mappers::convert::from_ros_string(msg.format),
    }
}

pub(crate) fn compressed_point_cloud_to_ros(bus: crate::foxglove_msgs::msg::v1::CompressedPointCloud) -> ros_env::foxglove_msgs::msg::CompressedPointCloud {
    ros_env::foxglove_msgs::msg::CompressedPointCloud {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
        format: crate::ros2_bridge::mappers::convert::to_ros_string(bus.format),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCompressedPointCloudMapper;

impl TypedTopicMapper for FoxgloveMsgsCompressedPointCloudMapper {
    type Ros = ros_env::foxglove_msgs::msg::CompressedPointCloud;
    type Bus = crate::foxglove_msgs::msg::v1::CompressedPointCloud;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(compressed_point_cloud_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(compressed_point_cloud_to_ros(msg))
    }
}
