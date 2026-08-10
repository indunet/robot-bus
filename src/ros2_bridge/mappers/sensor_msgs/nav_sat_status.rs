//! Mapper for `sensor_msgs/msg/NavSatStatus`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn nav_sat_status_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::NavSatStatus> {
    Ok(crate::sensor_msgs::msg::v1::NavSatStatus {
        status: read_i32(view, "status")?,
        service: read_u32(view, "service")?,
    })
}

pub(crate) fn nav_sat_status_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::NavSatStatus,
) -> Result<()> {
    write_i32(view, "status", bus.status)?;
    write_u32(view, "service", bus.service)?;
    Ok(())
}

pub(crate) fn nav_sat_status_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::NavSatStatus> {
    nav_sat_status_from_view(&msg.view())
}

pub(crate) fn nav_sat_status_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::NavSatStatus,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/NavSatStatus")?;
    nav_sat_status_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsNavSatStatusMapper;
impl TopicMapper for SensorMsgsNavSatStatusMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/NavSatStatus"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(nav_sat_status_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::NavSatStatus as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/NavSatStatus: {e}")))?;
        nav_sat_status_bus_to_dyn(&bus)
    }
}
