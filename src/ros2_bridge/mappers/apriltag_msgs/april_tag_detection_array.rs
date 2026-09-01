//! Typed mapper for `apriltag_msgs/msg/AprilTagDetectionArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn april_tag_detection_array_to_bus(msg: ros_env::apriltag_msgs::msg::AprilTagDetectionArray) -> crate::apriltag_msgs::msg::v1::AprilTagDetectionArray {
    crate::apriltag_msgs::msg::v1::AprilTagDetectionArray {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        detections: msg.detections.into_iter().map(crate::ros2_bridge::mappers::apriltag_msgs::april_tag_detection::april_tag_detection_to_bus).collect(),
    }
}

pub(crate) fn april_tag_detection_array_to_ros(bus: crate::apriltag_msgs::msg::v1::AprilTagDetectionArray) -> ros_env::apriltag_msgs::msg::AprilTagDetectionArray {
    ros_env::apriltag_msgs::msg::AprilTagDetectionArray {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        detections: bus.detections.into_iter().map(crate::ros2_bridge::mappers::apriltag_msgs::april_tag_detection::april_tag_detection_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ApriltagMsgsAprilTagDetectionArrayMapper;

impl TypedTopicMapper for ApriltagMsgsAprilTagDetectionArrayMapper {
    type Ros = ros_env::apriltag_msgs::msg::AprilTagDetectionArray;
    type Bus = crate::apriltag_msgs::msg::v1::AprilTagDetectionArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(april_tag_detection_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(april_tag_detection_array_to_ros(msg))
    }
}
