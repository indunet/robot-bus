//! Typed mapper for `foxglove_msgs/msg/PointCloud`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_cloud_to_bus(msg: ros_env::foxglove_msgs::msg::PointCloud) -> crate::foxglove_msgs::msg::v1::PointCloud {
    crate::foxglove_msgs::msg::v1::PointCloud {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        point_stride: msg.point_stride,
        fields: msg.fields.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::packed_element_field::packed_element_field_to_bus).collect(),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn point_cloud_to_ros(bus: crate::foxglove_msgs::msg::v1::PointCloud) -> ros_env::foxglove_msgs::msg::PointCloud {
    ros_env::foxglove_msgs::msg::PointCloud {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        point_stride: bus.point_stride,
        fields: bus.fields.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::packed_element_field::packed_element_field_to_ros).collect(),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPointCloudMapper;

impl TypedTopicMapper for FoxgloveMsgsPointCloudMapper {
    type Ros = ros_env::foxglove_msgs::msg::PointCloud;
    type Bus = crate::foxglove_msgs::msg::v1::PointCloud;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PointCloud"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_cloud_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_cloud_to_ros(msg))
    }
}
