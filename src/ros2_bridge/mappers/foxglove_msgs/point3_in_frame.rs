//! Typed mapper for `foxglove_msgs/msg/Point3InFrame`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point3_in_frame_to_bus(msg: ros_env::foxglove_msgs::msg::Point3InFrame) -> crate::foxglove_msgs::msg::v1::Point3InFrame {
    crate::foxglove_msgs::msg::v1::Point3InFrame {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        point: Some(crate::ros2_bridge::mappers::foxglove_msgs::point3::point3_to_bus(msg.point)),
    }
}

pub(crate) fn point3_in_frame_to_ros(bus: crate::foxglove_msgs::msg::v1::Point3InFrame) -> ros_env::foxglove_msgs::msg::Point3InFrame {
    ros_env::foxglove_msgs::msg::Point3InFrame {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        point: crate::ros2_bridge::mappers::foxglove_msgs::point3::point3_to_ros(bus.point.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPoint3InFrameMapper;

impl TypedTopicMapper for FoxgloveMsgsPoint3InFrameMapper {
    type Ros = ros_env::foxglove_msgs::msg::Point3InFrame;
    type Bus = crate::foxglove_msgs::msg::v1::Point3InFrame;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point3_in_frame_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point3_in_frame_to_ros(msg))
    }
}
