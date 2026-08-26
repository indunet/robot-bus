//! Mapper for `apriltag_msgs/msg/AprilTagDetection`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn april_tag_detection_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::apriltag_msgs::msg::v1::AprilTagDetection> {
    Ok(crate::apriltag_msgs::msg::v1::AprilTagDetection {
        family: read_string(view, "family")?,
        id: read_i32(view, "id")?,
        hamming: read_i32(view, "hamming")?,
        goodness: read_f32(view, "goodness")?,
        decision_margin: read_f32(view, "decision_margin")?,
        centre: nested_view(view, "centre")?
            .as_ref()
            .map(super::point::point_from_view)
            .transpose()?,
        corners: read_message_seq(view, "corners", super::point::point_from_view)?,
        homography: read_f64_seq(view, "homography")?,
    })
}

pub(crate) fn april_tag_detection_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::apriltag_msgs::msg::v1::AprilTagDetection,
) -> Result<()> {
    write_string(view, "family", &bus.family)?;
    write_i32(view, "id", bus.id)?;
    write_i32(view, "hamming", bus.hamming)?;
    write_f32(view, "goodness", bus.goodness)?;
    write_f32(view, "decision_margin", bus.decision_margin)?;
    if let Some(v) = &bus.centre {
        with_nested_mut(view, "centre", |nested| {
            super::point::point_write(nested, v)
        })?;
    }
    write_message_seq(view, "corners", &bus.corners, super::point::point_write)?;
    write_f64_seq(view, "homography", &bus.homography)?;
    Ok(())
}

pub(crate) fn april_tag_detection_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::apriltag_msgs::msg::v1::AprilTagDetection> {
    april_tag_detection_from_view(&msg.view())
}

pub(crate) fn april_tag_detection_bus_to_dyn(
    bus: &crate::apriltag_msgs::msg::v1::AprilTagDetection,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("apriltag_msgs/msg/AprilTagDetection")?;
    april_tag_detection_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ApriltagMsgsAprilTagDetectionMapper;
impl TopicMapper for ApriltagMsgsAprilTagDetectionMapper {
    fn type_name(&self) -> &'static str {
        "apriltag_msgs/msg/AprilTagDetection"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(april_tag_detection_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::apriltag_msgs::msg::v1::AprilTagDetection as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode apriltag_msgs/msg/AprilTagDetection: {e}"))
                })?;
        april_tag_detection_bus_to_dyn(&bus)
    }
}
