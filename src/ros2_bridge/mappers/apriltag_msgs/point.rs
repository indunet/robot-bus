//! Typed mapper for `apriltag_msgs/msg/Point`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_to_bus(msg: ros_env::apriltag_msgs::msg::Point) -> crate::apriltag_msgs::msg::v1::Point {
    crate::apriltag_msgs::msg::v1::Point {
        x: msg.x,
        y: msg.y,
    }
}

pub(crate) fn point_to_ros(bus: crate::apriltag_msgs::msg::v1::Point) -> ros_env::apriltag_msgs::msg::Point {
    ros_env::apriltag_msgs::msg::Point {
        x: bus.x,
        y: bus.y,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ApriltagMsgsPointMapper;

impl TypedTopicMapper for ApriltagMsgsPointMapper {
    type Ros = ros_env::apriltag_msgs::msg::Point;
    type Bus = crate::apriltag_msgs::msg::v1::Point;

    fn type_name(&self) -> &'static str {
        "apriltag_msgs/msg/Point"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_to_ros(msg))
    }
}
