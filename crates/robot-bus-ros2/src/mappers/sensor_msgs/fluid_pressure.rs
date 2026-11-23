//! Typed mapper for `sensor_msgs/msg/FluidPressure`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn fluid_pressure_to_bus(
    msg: ros_env::sensor_msgs::msg::FluidPressure,
) -> robot_bus::sensor_msgs::msg::v1::FluidPressure {
    robot_bus::sensor_msgs::msg::v1::FluidPressure {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        fluid_pressure: msg.fluid_pressure,
        variance: msg.variance,
    }
}

pub(crate) fn fluid_pressure_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::FluidPressure,
) -> ros_env::sensor_msgs::msg::FluidPressure {
    ros_env::sensor_msgs::msg::FluidPressure {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        fluid_pressure: bus.fluid_pressure,
        variance: bus.variance,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsFluidPressureMapper;

impl TypedTopicMapper for SensorMsgsFluidPressureMapper {
    type Ros = ros_env::sensor_msgs::msg::FluidPressure;
    type Bus = robot_bus::sensor_msgs::msg::v1::FluidPressure;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(fluid_pressure_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(fluid_pressure_to_ros(msg))
    }
}
