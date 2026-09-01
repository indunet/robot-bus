//! Typed mapper for `foxglove_msgs/msg/FrameTransforms`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn frame_transforms_to_bus(msg: ros_env::foxglove_msgs::msg::FrameTransforms) -> crate::foxglove_msgs::msg::v1::FrameTransforms {
    crate::foxglove_msgs::msg::v1::FrameTransforms {
        transforms: msg.transforms.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::frame_transform::frame_transform_to_bus).collect(),
    }
}

pub(crate) fn frame_transforms_to_ros(bus: crate::foxglove_msgs::msg::v1::FrameTransforms) -> ros_env::foxglove_msgs::msg::FrameTransforms {
    ros_env::foxglove_msgs::msg::FrameTransforms {
        transforms: bus.transforms.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::frame_transform::frame_transform_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsFrameTransformsMapper;

impl TypedTopicMapper for FoxgloveMsgsFrameTransformsMapper {
    type Ros = ros_env::foxglove_msgs::msg::FrameTransforms;
    type Bus = crate::foxglove_msgs::msg::v1::FrameTransforms;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(frame_transforms_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(frame_transforms_to_ros(msg))
    }
}
