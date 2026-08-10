//! Mapper for `sensor_msgs/msg/BatteryState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn battery_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::BatteryState> {
    Ok(crate::sensor_msgs::msg::v1::BatteryState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        voltage: read_f32(view, "voltage")?,
        current: read_f32(view, "current")?,
        charge: read_f32(view, "charge")?,
        capacity: read_f32(view, "capacity")?,
        design_capacity: read_f32(view, "design_capacity")?,
        percentage: read_f32(view, "percentage")?,
        power_supply_status: read_u32(view, "power_supply_status")?,
        power_supply_health: read_u32(view, "power_supply_health")?,
        power_supply_technology: read_u32(view, "power_supply_technology")?,
        present: read_bool(view, "present")?,
        cell_voltage: read_f32_seq(view, "cell_voltage")?,
        cell_temperature: read_f32_seq(view, "cell_temperature")?,
        location: read_string(view, "location")?,
        serial_number: read_string(view, "serial_number")?,
        temperature: read_f32(view, "temperature")?,
    })
}

pub(crate) fn battery_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::BatteryState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f32(view, "voltage", bus.voltage)?;
    write_f32(view, "current", bus.current)?;
    write_f32(view, "charge", bus.charge)?;
    write_f32(view, "capacity", bus.capacity)?;
    write_f32(view, "design_capacity", bus.design_capacity)?;
    write_f32(view, "percentage", bus.percentage)?;
    write_u32(view, "power_supply_status", bus.power_supply_status)?;
    write_u32(view, "power_supply_health", bus.power_supply_health)?;
    write_u32(view, "power_supply_technology", bus.power_supply_technology)?;
    write_bool(view, "present", bus.present)?;
    write_f32_seq(view, "cell_voltage", &bus.cell_voltage)?;
    write_f32_seq(view, "cell_temperature", &bus.cell_temperature)?;
    write_string(view, "location", &bus.location)?;
    write_string(view, "serial_number", &bus.serial_number)?;
    write_f32(view, "temperature", bus.temperature)?;
    Ok(())
}

pub(crate) fn battery_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::BatteryState> {
    battery_state_from_view(&msg.view())
}

pub(crate) fn battery_state_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::BatteryState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/BatteryState")?;
    battery_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsBatteryStateMapper;
impl TopicMapper for SensorMsgsBatteryStateMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/BatteryState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(battery_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::BatteryState as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/BatteryState: {e}")))?;
        battery_state_bus_to_dyn(&bus)
    }
}
