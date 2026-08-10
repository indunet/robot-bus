//! Mapper for `foxglove_msgs/msg/CameraCalibration`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn camera_calibration_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CameraCalibration> {
    Ok(crate::foxglove_msgs::msg::v1::CameraCalibration {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        width: read_u32(view, "width")?,
        height: read_u32(view, "height")?,
        distortion_model: read_string(view, "distortion_model")?,
        d: read_f64_seq(view, "d")?,
        k: read_f64_seq(view, "k")?,
        r: read_f64_seq(view, "r")?,
        p: read_f64_seq(view, "p")?,
    })
}

pub(crate) fn camera_calibration_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CameraCalibration,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_u32(view, "width", bus.width)?;
    write_u32(view, "height", bus.height)?;
    write_string(view, "distortion_model", &bus.distortion_model)?;
    write_f64_seq(view, "d", &bus.d)?;
    write_f64_seq(view, "k", &bus.k)?;
    write_f64_seq(view, "r", &bus.r)?;
    write_f64_seq(view, "p", &bus.p)?;
    Ok(())
}

pub(crate) fn camera_calibration_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CameraCalibration> {
    camera_calibration_from_view(&msg.view())
}

pub(crate) fn camera_calibration_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CameraCalibration,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CameraCalibration")?;
    camera_calibration_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCameraCalibrationMapper;
impl TopicMapper for FoxgloveMsgsCameraCalibrationMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CameraCalibration"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(camera_calibration_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::CameraCalibration as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode foxglove_msgs/msg/CameraCalibration: {e}"))
                })?;
        camera_calibration_bus_to_dyn(&bus)
    }
}
