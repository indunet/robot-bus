//! Typed mapper for `apriltag_msgs/msg/AprilTagDetection`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn april_tag_detection_to_bus(msg: ros_env::apriltag_msgs::msg::AprilTagDetection) -> crate::apriltag_msgs::msg::v1::AprilTagDetection {
    crate::apriltag_msgs::msg::v1::AprilTagDetection {
        family: crate::ros2_bridge::mappers::convert::from_ros_string(msg.family),
        id: msg.id,
        hamming: msg.hamming,
        goodness: msg.goodness,
        decision_margin: msg.decision_margin,
        centre: Some(crate::ros2_bridge::mappers::apriltag_msgs::point::point_to_bus(msg.centre)),
        corners: msg.corners.into_iter().map(crate::ros2_bridge::mappers::apriltag_msgs::point::point_to_bus).collect(),
        homography: crate::ros2_bridge::mappers::convert::f64_seq(msg.homography),
    }
}

pub(crate) fn april_tag_detection_to_ros(bus: crate::apriltag_msgs::msg::v1::AprilTagDetection) -> ros_env::apriltag_msgs::msg::AprilTagDetection {
    ros_env::apriltag_msgs::msg::AprilTagDetection {
        family: crate::ros2_bridge::mappers::convert::to_ros_string(bus.family),
        id: bus.id,
        hamming: bus.hamming,
        goodness: bus.goodness,
        decision_margin: bus.decision_margin,
        centre: crate::ros2_bridge::mappers::apriltag_msgs::point::point_to_ros(bus.centre.unwrap_or_default()),
        corners: bus.corners.into_iter().map(crate::ros2_bridge::mappers::apriltag_msgs::point::point_to_ros).collect(),
        homography: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.homography),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ApriltagMsgsAprilTagDetectionMapper;

impl TypedTopicMapper for ApriltagMsgsAprilTagDetectionMapper {
    type Ros = ros_env::apriltag_msgs::msg::AprilTagDetection;
    type Bus = crate::apriltag_msgs::msg::v1::AprilTagDetection;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(april_tag_detection_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(april_tag_detection_to_ros(msg))
    }
}
