//! Typed mapper for `foxglove_msgs/msg/Color`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn color_to_bus(msg: ros_env::foxglove_msgs::msg::Color) -> crate::foxglove_msgs::msg::v1::Color {
    crate::foxglove_msgs::msg::v1::Color {
        r: msg.r,
        g: msg.g,
        b: msg.b,
        a: msg.a,
    }
}

pub(crate) fn color_to_ros(bus: crate::foxglove_msgs::msg::v1::Color) -> ros_env::foxglove_msgs::msg::Color {
    ros_env::foxglove_msgs::msg::Color {
        r: bus.r,
        g: bus.g,
        b: bus.b,
        a: bus.a,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsColorMapper;

impl TypedTopicMapper for FoxgloveMsgsColorMapper {
    type Ros = ros_env::foxglove_msgs::msg::Color;
    type Bus = crate::foxglove_msgs::msg::v1::Color;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Color"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(color_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(color_to_ros(msg))
    }
}
