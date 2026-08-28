//! Typed mapper for `sensor_msgs/msg/Joy`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joy_to_bus(msg: ros_env::sensor_msgs::msg::Joy) -> crate::sensor_msgs::msg::v1::Joy {
    crate::sensor_msgs::msg::v1::Joy {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        axes: crate::ros2_bridge::mappers::convert::f32_seq(msg.axes),
        buttons: crate::ros2_bridge::mappers::convert::i32_seq(msg.buttons),
    }
}

pub(crate) fn joy_to_ros(bus: crate::sensor_msgs::msg::v1::Joy) -> ros_env::sensor_msgs::msg::Joy {
    ros_env::sensor_msgs::msg::Joy {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        axes: bus.axes,
        buttons: bus.buttons,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJoyMapper;

impl TypedTopicMapper for SensorMsgsJoyMapper {
    type Ros = ros_env::sensor_msgs::msg::Joy;
    type Bus = crate::sensor_msgs::msg::v1::Joy;

    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Joy"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joy_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joy_to_ros(msg))
    }
}
