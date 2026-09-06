//! Typed mapper for `sensor_msgs/msg/Illuminance`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn illuminance_to_bus(
    msg: ros_env::sensor_msgs::msg::Illuminance,
) -> crate::sensor_msgs::msg::v1::Illuminance {
    crate::sensor_msgs::msg::v1::Illuminance {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        illuminance: msg.illuminance,
        variance: msg.variance,
    }
}

pub(crate) fn illuminance_to_ros(
    bus: crate::sensor_msgs::msg::v1::Illuminance,
) -> ros_env::sensor_msgs::msg::Illuminance {
    ros_env::sensor_msgs::msg::Illuminance {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        illuminance: bus.illuminance,
        variance: bus.variance,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsIlluminanceMapper;

impl TypedTopicMapper for SensorMsgsIlluminanceMapper {
    type Ros = ros_env::sensor_msgs::msg::Illuminance;
    type Bus = crate::sensor_msgs::msg::v1::Illuminance;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(illuminance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(illuminance_to_ros(msg))
    }
}
