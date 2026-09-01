//! Typed mapper for `foxglove_msgs/msg/FrameTransform`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn frame_transform_to_bus(msg: ros_env::foxglove_msgs::msg::FrameTransform) -> crate::foxglove_msgs::msg::v1::FrameTransform {
    crate::foxglove_msgs::msg::v1::FrameTransform {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        parent_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.parent_frame_id),
        child_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.child_frame_id),
        translation: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus(msg.translation)),
        rotation: Some(crate::ros2_bridge::mappers::foxglove_msgs::quaternion::quaternion_to_bus(msg.rotation)),
    }
}

pub(crate) fn frame_transform_to_ros(bus: crate::foxglove_msgs::msg::v1::FrameTransform) -> ros_env::foxglove_msgs::msg::FrameTransform {
    ros_env::foxglove_msgs::msg::FrameTransform {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        parent_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.parent_frame_id),
        child_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.child_frame_id),
        translation: crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros(bus.translation.unwrap_or_default()),
        rotation: crate::ros2_bridge::mappers::foxglove_msgs::quaternion::quaternion_to_ros(bus.rotation.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsFrameTransformMapper;

impl TypedTopicMapper for FoxgloveMsgsFrameTransformMapper {
    type Ros = ros_env::foxglove_msgs::msg::FrameTransform;
    type Bus = crate::foxglove_msgs::msg::v1::FrameTransform;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(frame_transform_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(frame_transform_to_ros(msg))
    }
}
