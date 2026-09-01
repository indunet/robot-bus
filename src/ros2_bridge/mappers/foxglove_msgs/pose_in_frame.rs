//! Typed mapper for `foxglove_msgs/msg/PoseInFrame`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn pose_in_frame_to_bus(msg: ros_env::foxglove_msgs::msg::PoseInFrame) -> crate::foxglove_msgs::msg::v1::PoseInFrame {
    crate::foxglove_msgs::msg::v1::PoseInFrame {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
    }
}

pub(crate) fn pose_in_frame_to_ros(bus: crate::foxglove_msgs::msg::v1::PoseInFrame) -> ros_env::foxglove_msgs::msg::PoseInFrame {
    ros_env::foxglove_msgs::msg::PoseInFrame {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPoseInFrameMapper;

impl TypedTopicMapper for FoxgloveMsgsPoseInFrameMapper {
    type Ros = ros_env::foxglove_msgs::msg::PoseInFrame;
    type Bus = crate::foxglove_msgs::msg::v1::PoseInFrame;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(pose_in_frame_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(pose_in_frame_to_ros(msg))
    }
}
