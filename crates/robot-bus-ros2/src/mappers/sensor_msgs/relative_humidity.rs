//! Typed mapper for `sensor_msgs/msg/RelativeHumidity`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn relative_humidity_to_bus(
    msg: ros_env::sensor_msgs::msg::RelativeHumidity,
) -> robot_bus::sensor_msgs::msg::v1::RelativeHumidity {
    robot_bus::sensor_msgs::msg::v1::RelativeHumidity {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        relative_humidity: msg.relative_humidity,
        variance: msg.variance,
    }
}

pub(crate) fn relative_humidity_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::RelativeHumidity,
) -> ros_env::sensor_msgs::msg::RelativeHumidity {
    ros_env::sensor_msgs::msg::RelativeHumidity {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        relative_humidity: bus.relative_humidity,
        variance: bus.variance,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsRelativeHumidityMapper;

impl TypedTopicMapper for SensorMsgsRelativeHumidityMapper {
    type Ros = ros_env::sensor_msgs::msg::RelativeHumidity;
    type Bus = robot_bus::sensor_msgs::msg::v1::RelativeHumidity;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(relative_humidity_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(relative_humidity_to_ros(msg))
    }
}
