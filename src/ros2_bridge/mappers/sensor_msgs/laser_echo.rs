//! Typed mapper for `sensor_msgs/msg/LaserEcho`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn laser_echo_to_bus(msg: ros_env::sensor_msgs::msg::LaserEcho) -> crate::sensor_msgs::msg::v1::LaserEcho {
    crate::sensor_msgs::msg::v1::LaserEcho {
        echoes: crate::ros2_bridge::mappers::convert::f32_seq(msg.echoes),
    }
}

pub(crate) fn laser_echo_to_ros(bus: crate::sensor_msgs::msg::v1::LaserEcho) -> ros_env::sensor_msgs::msg::LaserEcho {
    ros_env::sensor_msgs::msg::LaserEcho {
        echoes: bus.echoes,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsLaserEchoMapper;

impl TypedTopicMapper for SensorMsgsLaserEchoMapper {
    type Ros = ros_env::sensor_msgs::msg::LaserEcho;
    type Bus = crate::sensor_msgs::msg::v1::LaserEcho;

    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/LaserEcho"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(laser_echo_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(laser_echo_to_ros(msg))
    }
}
