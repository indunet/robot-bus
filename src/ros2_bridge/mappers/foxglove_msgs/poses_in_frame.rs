//! Typed mapper for `foxglove_msgs/msg/PosesInFrame`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn poses_in_frame_to_bus(msg: ros_env::foxglove_msgs::msg::PosesInFrame) -> crate::foxglove_msgs::msg::v1::PosesInFrame {
    crate::foxglove_msgs::msg::v1::PosesInFrame {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        poses: msg.poses.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus).collect(),
    }
}

pub(crate) fn poses_in_frame_to_ros(bus: crate::foxglove_msgs::msg::v1::PosesInFrame) -> ros_env::foxglove_msgs::msg::PosesInFrame {
    ros_env::foxglove_msgs::msg::PosesInFrame {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        poses: bus.poses.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPosesInFrameMapper;

impl TypedTopicMapper for FoxgloveMsgsPosesInFrameMapper {
    type Ros = ros_env::foxglove_msgs::msg::PosesInFrame;
    type Bus = crate::foxglove_msgs::msg::v1::PosesInFrame;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PosesInFrame"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(poses_in_frame_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(poses_in_frame_to_ros(msg))
    }
}
