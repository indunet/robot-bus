//! Mapper for `sensor_msgs/msg/Illuminance`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn illuminance_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Illuminance> {
    Ok(crate::sensor_msgs::msg::v1::Illuminance {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        illuminance: read_f64(view, "illuminance")?,
        variance: read_f64(view, "variance")?,
    })
}

pub(crate) fn illuminance_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Illuminance,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64(view, "illuminance", bus.illuminance)?;
    write_f64(view, "variance", bus.variance)?;
    Ok(())
}

pub(crate) fn illuminance_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Illuminance> {
    illuminance_from_view(&msg.view())
}

pub(crate) fn illuminance_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Illuminance,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Illuminance")?;
    illuminance_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsIlluminanceMapper;
impl TopicMapper for SensorMsgsIlluminanceMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Illuminance"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(illuminance_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Illuminance as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Illuminance: {e}")))?;
        illuminance_bus_to_dyn(&bus)
    }
}
