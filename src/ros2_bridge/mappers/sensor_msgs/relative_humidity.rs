//! Typed mapper for `sensor_msgs/msg/RelativeHumidity`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn relative_humidity_to_bus(msg: ros_env::sensor_msgs::msg::RelativeHumidity) -> crate::sensor_msgs::msg::v1::RelativeHumidity {
    crate::sensor_msgs::msg::v1::RelativeHumidity {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        relative_humidity: msg.relative_humidity,
        variance: msg.variance,
    }
}

pub(crate) fn relative_humidity_to_ros(bus: crate::sensor_msgs::msg::v1::RelativeHumidity) -> ros_env::sensor_msgs::msg::RelativeHumidity {
    ros_env::sensor_msgs::msg::RelativeHumidity {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        relative_humidity: bus.relative_humidity,
        variance: bus.variance,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsRelativeHumidityMapper;

impl TypedTopicMapper for SensorMsgsRelativeHumidityMapper {
    type Ros = ros_env::sensor_msgs::msg::RelativeHumidity;
    type Bus = crate::sensor_msgs::msg::v1::RelativeHumidity;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(relative_humidity_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(relative_humidity_to_ros(msg))
    }
}
