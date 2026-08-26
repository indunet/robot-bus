//! Mapper for `sensor_msgs/msg/LaserEcho`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn laser_echo_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::LaserEcho> {
    Ok(crate::sensor_msgs::msg::v1::LaserEcho {
        echoes: read_f32_seq(view, "echoes")?,
    })
}

pub(crate) fn laser_echo_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::LaserEcho,
) -> Result<()> {
    write_f32_seq(view, "echoes", &bus.echoes)?;
    Ok(())
}

pub(crate) fn laser_echo_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::LaserEcho> {
    laser_echo_from_view(&msg.view())
}

pub(crate) fn laser_echo_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::LaserEcho,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/LaserEcho")?;
    laser_echo_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsLaserEchoMapper;
impl TopicMapper for SensorMsgsLaserEchoMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/LaserEcho"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(laser_echo_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::LaserEcho as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/LaserEcho: {e}")))?;
        laser_echo_bus_to_dyn(&bus)
    }
}
