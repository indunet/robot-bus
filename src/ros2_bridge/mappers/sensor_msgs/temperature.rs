//! Typed mapper for `sensor_msgs/msg/Temperature`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn temperature_to_bus(msg: ros_env::sensor_msgs::msg::Temperature) -> crate::sensor_msgs::msg::v1::Temperature {
    crate::sensor_msgs::msg::v1::Temperature {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        temperature: msg.temperature,
        variance: msg.variance,
    }
}

pub(crate) fn temperature_to_ros(bus: crate::sensor_msgs::msg::v1::Temperature) -> ros_env::sensor_msgs::msg::Temperature {
    ros_env::sensor_msgs::msg::Temperature {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        temperature: bus.temperature,
        variance: bus.variance,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsTemperatureMapper;

impl TypedTopicMapper for SensorMsgsTemperatureMapper {
    type Ros = ros_env::sensor_msgs::msg::Temperature;
    type Bus = crate::sensor_msgs::msg::v1::Temperature;

    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Temperature"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(temperature_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(temperature_to_ros(msg))
    }
}
