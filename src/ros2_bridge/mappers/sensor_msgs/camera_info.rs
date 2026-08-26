//! Mapper for `sensor_msgs/msg/CameraInfo`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn camera_info_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::CameraInfo> {
    Ok(crate::sensor_msgs::msg::v1::CameraInfo {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        height: read_u32(view, "height")?,
        width: read_u32(view, "width")?,
        distortion_model: read_string(view, "distortion_model")?,
        d: read_f64_seq(view, "d")?,
        k: read_f64_seq(view, "k")?,
        r: read_f64_seq(view, "r")?,
        p: read_f64_seq(view, "p")?,
        binning_x: read_u32(view, "binning_x")?,
        binning_y: read_u32(view, "binning_y")?,
        roi: nested_view(view, "roi")?
            .as_ref()
            .map(super::region_of_interest::region_of_interest_from_view)
            .transpose()?,
    })
}

pub(crate) fn camera_info_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::CameraInfo,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "height", bus.height)?;
    write_u32(view, "width", bus.width)?;
    write_string(view, "distortion_model", &bus.distortion_model)?;
    write_f64_seq(view, "d", &bus.d)?;
    write_f64_seq(view, "k", &bus.k)?;
    write_f64_seq(view, "r", &bus.r)?;
    write_f64_seq(view, "p", &bus.p)?;
    write_u32(view, "binning_x", bus.binning_x)?;
    write_u32(view, "binning_y", bus.binning_y)?;
    if let Some(v) = &bus.roi {
        with_nested_mut(view, "roi", |nested| {
            super::region_of_interest::region_of_interest_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn camera_info_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::CameraInfo> {
    camera_info_from_view(&msg.view())
}

pub(crate) fn camera_info_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::CameraInfo,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/CameraInfo")?;
    camera_info_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsCameraInfoMapper;
impl TopicMapper for SensorMsgsCameraInfoMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/CameraInfo"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(camera_info_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::CameraInfo as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/CameraInfo: {e}")))?;
        camera_info_bus_to_dyn(&bus)
    }
}
