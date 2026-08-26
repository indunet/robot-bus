//! Mapper for `foxglove_msgs/msg/LaserScan`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn laser_scan_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::LaserScan> {
    Ok(crate::foxglove_msgs::msg::v1::LaserScan {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        start_angle: read_f64(view, "start_angle")?,
        end_angle: read_f64(view, "end_angle")?,
        ranges: read_f64_seq(view, "ranges")?,
        intensities: read_f64_seq(view, "intensities")?,
    })
}

pub(crate) fn laser_scan_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::LaserScan,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_f64(view, "start_angle", bus.start_angle)?;
    write_f64(view, "end_angle", bus.end_angle)?;
    write_f64_seq(view, "ranges", &bus.ranges)?;
    write_f64_seq(view, "intensities", &bus.intensities)?;
    Ok(())
}

pub(crate) fn laser_scan_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::LaserScan> {
    laser_scan_from_view(&msg.view())
}

pub(crate) fn laser_scan_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::LaserScan,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/LaserScan")?;
    laser_scan_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsLaserScanMapper;
impl TopicMapper for FoxgloveMsgsLaserScanMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LaserScan"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(laser_scan_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::LaserScan as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/LaserScan: {e}")))?;
        laser_scan_bus_to_dyn(&bus)
    }
}
