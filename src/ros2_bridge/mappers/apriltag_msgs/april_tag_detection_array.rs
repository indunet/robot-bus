//! Mapper for `apriltag_msgs/msg/AprilTagDetectionArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn april_tag_detection_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::apriltag_msgs::msg::v1::AprilTagDetectionArray> {
    Ok(crate::apriltag_msgs::msg::v1::AprilTagDetectionArray {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        detections: read_message_seq(view, "detections", super::april_tag_detection::april_tag_detection_from_view)?,
    })
}

pub(crate) fn april_tag_detection_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::apriltag_msgs::msg::v1::AprilTagDetectionArray,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "detections",
        &bus.detections,
        super::april_tag_detection::april_tag_detection_write,
    )?;
    Ok(())
}

pub(crate) fn april_tag_detection_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::apriltag_msgs::msg::v1::AprilTagDetectionArray> {
    april_tag_detection_array_from_view(&msg.view())
}

pub(crate) fn april_tag_detection_array_bus_to_dyn(
    bus: &crate::apriltag_msgs::msg::v1::AprilTagDetectionArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("apriltag_msgs/msg/AprilTagDetectionArray")?;
    april_tag_detection_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ApriltagMsgsAprilTagDetectionArrayMapper;
impl TopicMapper for ApriltagMsgsAprilTagDetectionArrayMapper {
    fn type_name(&self) -> &'static str {
        "apriltag_msgs/msg/AprilTagDetectionArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(april_tag_detection_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::apriltag_msgs::msg::v1::AprilTagDetectionArray as ProstMessage>::decode(
            payload,
        )
        .map_err(|e| {
            BusError::Protocol(format!(
                "decode apriltag_msgs/msg/AprilTagDetectionArray: {e}"
            ))
        })?;
        april_tag_detection_array_bus_to_dyn(&bus)
    }
}
