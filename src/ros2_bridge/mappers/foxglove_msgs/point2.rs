//! Typed mapper for `foxglove_msgs/msg/Point2`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point2_to_bus(msg: ros_env::foxglove_msgs::msg::Point2) -> crate::foxglove_msgs::msg::v1::Point2 {
    crate::foxglove_msgs::msg::v1::Point2 {
        x: msg.x,
        y: msg.y,
    }
}

pub(crate) fn point2_to_ros(bus: crate::foxglove_msgs::msg::v1::Point2) -> ros_env::foxglove_msgs::msg::Point2 {
    ros_env::foxglove_msgs::msg::Point2 {
        x: bus.x,
        y: bus.y,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPoint2Mapper;

impl TypedTopicMapper for FoxgloveMsgsPoint2Mapper {
    type Ros = ros_env::foxglove_msgs::msg::Point2;
    type Bus = crate::foxglove_msgs::msg::v1::Point2;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Point2"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point2_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point2_to_ros(msg))
    }
}
