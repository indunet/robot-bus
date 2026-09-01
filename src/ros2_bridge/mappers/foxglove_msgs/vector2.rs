//! Typed mapper for `foxglove_msgs/msg/Vector2`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn vector2_to_bus(msg: ros_env::foxglove_msgs::msg::Vector2) -> crate::foxglove_msgs::msg::v1::Vector2 {
    crate::foxglove_msgs::msg::v1::Vector2 {
        x: msg.x,
        y: msg.y,
    }
}

pub(crate) fn vector2_to_ros(bus: crate::foxglove_msgs::msg::v1::Vector2) -> ros_env::foxglove_msgs::msg::Vector2 {
    ros_env::foxglove_msgs::msg::Vector2 {
        x: bus.x,
        y: bus.y,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsVector2Mapper;

impl TypedTopicMapper for FoxgloveMsgsVector2Mapper {
    type Ros = ros_env::foxglove_msgs::msg::Vector2;
    type Bus = crate::foxglove_msgs::msg::v1::Vector2;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(vector2_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(vector2_to_ros(msg))
    }
}
