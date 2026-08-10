//! Mapper for `sensor_msgs/msg/NavSatFix`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn nav_sat_fix_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::NavSatFix> {
    Ok(crate::sensor_msgs::msg::v1::NavSatFix {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        status: nested_view(view, "status")?
            .as_ref()
            .map(super::nav_sat_status::nav_sat_status_from_view)
            .transpose()?,
        latitude: read_f64(view, "latitude")?,
        longitude: read_f64(view, "longitude")?,
        altitude: read_f64(view, "altitude")?,
        position_covariance: read_f64_seq(view, "position_covariance")?,
        position_covariance_type: read_u32(view, "position_covariance_type")?,
    })
}

pub(crate) fn nav_sat_fix_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::NavSatFix,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.status {
        with_nested_mut(view, "status", |nested| super::nav_sat_status::nav_sat_status_write(nested, v))?;
    }
    write_f64(view, "latitude", bus.latitude)?;
    write_f64(view, "longitude", bus.longitude)?;
    write_f64(view, "altitude", bus.altitude)?;
    write_f64_seq(view, "position_covariance", &bus.position_covariance)?;
    write_u32(
        view,
        "position_covariance_type",
        bus.position_covariance_type,
    )?;
    Ok(())
}

pub(crate) fn nav_sat_fix_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::NavSatFix> {
    nav_sat_fix_from_view(&msg.view())
}

pub(crate) fn nav_sat_fix_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::NavSatFix,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/NavSatFix")?;
    nav_sat_fix_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsNavSatFixMapper;
impl TopicMapper for SensorMsgsNavSatFixMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/NavSatFix"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(nav_sat_fix_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::NavSatFix as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/NavSatFix: {e}")))?;
        nav_sat_fix_bus_to_dyn(&bus)
    }
}
