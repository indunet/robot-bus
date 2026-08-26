//! Mapper for `sensor_msgs/msg/FluidPressure`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn fluid_pressure_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::FluidPressure> {
    Ok(crate::sensor_msgs::msg::v1::FluidPressure {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        fluid_pressure: read_f64(view, "fluid_pressure")?,
        variance: read_f64(view, "variance")?,
    })
}

pub(crate) fn fluid_pressure_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::FluidPressure,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64(view, "fluid_pressure", bus.fluid_pressure)?;
    write_f64(view, "variance", bus.variance)?;
    Ok(())
}

pub(crate) fn fluid_pressure_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::FluidPressure> {
    fluid_pressure_from_view(&msg.view())
}

pub(crate) fn fluid_pressure_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::FluidPressure,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/FluidPressure")?;
    fluid_pressure_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsFluidPressureMapper;
impl TopicMapper for SensorMsgsFluidPressureMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/FluidPressure"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(fluid_pressure_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::FluidPressure as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode sensor_msgs/msg/FluidPressure: {e}"))
            })?;
        fluid_pressure_bus_to_dyn(&bus)
    }
}
