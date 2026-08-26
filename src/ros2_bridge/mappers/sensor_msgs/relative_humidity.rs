//! Mapper for `sensor_msgs/msg/RelativeHumidity`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn relative_humidity_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::RelativeHumidity> {
    Ok(crate::sensor_msgs::msg::v1::RelativeHumidity {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        relative_humidity: read_f64(view, "relative_humidity")?,
        variance: read_f64(view, "variance")?,
    })
}

pub(crate) fn relative_humidity_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::RelativeHumidity,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64(view, "relative_humidity", bus.relative_humidity)?;
    write_f64(view, "variance", bus.variance)?;
    Ok(())
}

pub(crate) fn relative_humidity_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::RelativeHumidity> {
    relative_humidity_from_view(&msg.view())
}

pub(crate) fn relative_humidity_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::RelativeHumidity,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/RelativeHumidity")?;
    relative_humidity_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsRelativeHumidityMapper;
impl TopicMapper for SensorMsgsRelativeHumidityMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/RelativeHumidity"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(relative_humidity_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::RelativeHumidity as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode sensor_msgs/msg/RelativeHumidity: {e}"))
            })?;
        relative_humidity_bus_to_dyn(&bus)
    }
}
