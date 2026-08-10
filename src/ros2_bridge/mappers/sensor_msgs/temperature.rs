//! Mapper for `sensor_msgs/msg/Temperature`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn temperature_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Temperature> {
    Ok(crate::sensor_msgs::msg::v1::Temperature {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        temperature: read_f64(view, "temperature")?,
        variance: read_f64(view, "variance")?,
    })
}

pub(crate) fn temperature_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Temperature,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64(view, "temperature", bus.temperature)?;
    write_f64(view, "variance", bus.variance)?;
    Ok(())
}

pub(crate) fn temperature_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Temperature> {
    temperature_from_view(&msg.view())
}

pub(crate) fn temperature_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Temperature,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Temperature")?;
    temperature_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsTemperatureMapper;
impl TopicMapper for SensorMsgsTemperatureMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Temperature"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(temperature_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Temperature as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Temperature: {e}")))?;
        temperature_bus_to_dyn(&bus)
    }
}
