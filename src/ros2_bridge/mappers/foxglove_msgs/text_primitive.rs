//! Typed mapper for `foxglove_msgs/msg/TextPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn text_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::TextPrimitive) -> crate::foxglove_msgs::msg::v1::TextPrimitive {
    crate::foxglove_msgs::msg::v1::TextPrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        billboard: msg.billboard,
        font_size: msg.font_size,
        scale_invariant: msg.scale_invariant,
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
        text: crate::ros2_bridge::mappers::convert::from_ros_string(msg.text),
    }
}

pub(crate) fn text_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::TextPrimitive) -> ros_env::foxglove_msgs::msg::TextPrimitive {
    ros_env::foxglove_msgs::msg::TextPrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        billboard: bus.billboard,
        font_size: bus.font_size,
        scale_invariant: bus.scale_invariant,
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
        text: crate::ros2_bridge::mappers::convert::to_ros_string(bus.text),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsTextPrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsTextPrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::TextPrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::TextPrimitive;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(text_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(text_primitive_to_ros(msg))
    }
}
