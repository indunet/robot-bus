//! Typed mapper for `sensor_msgs/msg/BatteryState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn battery_state_to_bus(
    msg: ros_env::sensor_msgs::msg::BatteryState,
) -> crate::sensor_msgs::msg::v1::BatteryState {
    crate::sensor_msgs::msg::v1::BatteryState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        voltage: msg.voltage,
        current: msg.current,
        charge: msg.charge,
        capacity: msg.capacity,
        design_capacity: msg.design_capacity,
        percentage: msg.percentage,
        power_supply_status: msg.power_supply_status.into(),
        power_supply_health: msg.power_supply_health.into(),
        power_supply_technology: msg.power_supply_technology.into(),
        present: msg.present,
        cell_voltage: crate::ros2_bridge::mappers::convert::f32_seq(msg.cell_voltage),
        cell_temperature: crate::ros2_bridge::mappers::convert::f32_seq(msg.cell_temperature),
        location: crate::ros2_bridge::mappers::convert::from_ros_string(msg.location),
        serial_number: crate::ros2_bridge::mappers::convert::from_ros_string(msg.serial_number),
        temperature: msg.temperature,
    }
}

pub(crate) fn battery_state_to_ros(
    bus: crate::sensor_msgs::msg::v1::BatteryState,
) -> ros_env::sensor_msgs::msg::BatteryState {
    ros_env::sensor_msgs::msg::BatteryState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        voltage: bus.voltage,
        current: bus.current,
        charge: bus.charge,
        capacity: bus.capacity,
        design_capacity: bus.design_capacity,
        percentage: bus.percentage,
        power_supply_status: bus.power_supply_status as _,
        power_supply_health: bus.power_supply_health as _,
        power_supply_technology: bus.power_supply_technology as _,
        present: bus.present,
        cell_voltage: bus.cell_voltage,
        cell_temperature: bus.cell_temperature,
        location: crate::ros2_bridge::mappers::convert::to_ros_string(bus.location),
        serial_number: crate::ros2_bridge::mappers::convert::to_ros_string(bus.serial_number),
        temperature: bus.temperature,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsBatteryStateMapper;

impl TypedTopicMapper for SensorMsgsBatteryStateMapper {
    type Ros = ros_env::sensor_msgs::msg::BatteryState;
    type Bus = crate::sensor_msgs::msg::v1::BatteryState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(battery_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(battery_state_to_ros(msg))
    }
}
