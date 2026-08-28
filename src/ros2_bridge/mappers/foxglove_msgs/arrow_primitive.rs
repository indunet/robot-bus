//! Typed mapper for `foxglove_msgs/msg/ArrowPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn arrow_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::ArrowPrimitive) -> crate::foxglove_msgs::msg::v1::ArrowPrimitive {
    crate::foxglove_msgs::msg::v1::ArrowPrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        shaft_length: msg.shaft_length,
        shaft_diameter: msg.shaft_diameter,
        head_length: msg.head_length,
        head_diameter: msg.head_diameter,
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
    }
}

pub(crate) fn arrow_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::ArrowPrimitive) -> ros_env::foxglove_msgs::msg::ArrowPrimitive {
    ros_env::foxglove_msgs::msg::ArrowPrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        shaft_length: bus.shaft_length,
        shaft_diameter: bus.shaft_diameter,
        head_length: bus.head_length,
        head_diameter: bus.head_diameter,
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsArrowPrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsArrowPrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::ArrowPrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::ArrowPrimitive;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/ArrowPrimitive"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(arrow_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(arrow_primitive_to_ros(msg))
    }
}
