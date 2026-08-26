//! Mapper for `sensor_msgs/msg/MultiEchoLaserScan`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn multi_echo_laser_scan_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::MultiEchoLaserScan> {
    Ok(crate::sensor_msgs::msg::v1::MultiEchoLaserScan {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        angle_min: read_f32(view, "angle_min")?,
        angle_max: read_f32(view, "angle_max")?,
        angle_increment: read_f32(view, "angle_increment")?,
        time_increment: read_f32(view, "time_increment")?,
        scan_time: read_f32(view, "scan_time")?,
        range_min: read_f32(view, "range_min")?,
        range_max: read_f32(view, "range_max")?,
        ranges: read_message_seq(view, "ranges", super::laser_echo::laser_echo_from_view)?,
        intensities: read_message_seq(
            view,
            "intensities",
            super::laser_echo::laser_echo_from_view,
        )?,
    })
}

pub(crate) fn multi_echo_laser_scan_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::MultiEchoLaserScan,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f32(view, "angle_min", bus.angle_min)?;
    write_f32(view, "angle_max", bus.angle_max)?;
    write_f32(view, "angle_increment", bus.angle_increment)?;
    write_f32(view, "time_increment", bus.time_increment)?;
    write_f32(view, "scan_time", bus.scan_time)?;
    write_f32(view, "range_min", bus.range_min)?;
    write_f32(view, "range_max", bus.range_max)?;
    write_message_seq(
        view,
        "ranges",
        &bus.ranges,
        super::laser_echo::laser_echo_write,
    )?;
    write_message_seq(
        view,
        "intensities",
        &bus.intensities,
        super::laser_echo::laser_echo_write,
    )?;
    Ok(())
}

pub(crate) fn multi_echo_laser_scan_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::MultiEchoLaserScan> {
    multi_echo_laser_scan_from_view(&msg.view())
}

pub(crate) fn multi_echo_laser_scan_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::MultiEchoLaserScan,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/MultiEchoLaserScan")?;
    multi_echo_laser_scan_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsMultiEchoLaserScanMapper;
impl TopicMapper for SensorMsgsMultiEchoLaserScanMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/MultiEchoLaserScan"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_echo_laser_scan_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::sensor_msgs::msg::v1::MultiEchoLaserScan as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode sensor_msgs/msg/MultiEchoLaserScan: {e}"))
                })?;
        multi_echo_laser_scan_bus_to_dyn(&bus)
    }
}
