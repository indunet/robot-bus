//! Typed mapper for `sensor_msgs/msg/Joy`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn joy_to_bus(msg: ros_env::sensor_msgs::msg::Joy) -> robot_bus::sensor_msgs::msg::v1::Joy {
    robot_bus::sensor_msgs::msg::v1::Joy {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        axes: crate::mappers::convert::f32_seq(msg.axes),
        buttons: crate::mappers::convert::i32_seq(msg.buttons),
    }
}

pub(crate) fn joy_to_ros(bus: robot_bus::sensor_msgs::msg::v1::Joy) -> ros_env::sensor_msgs::msg::Joy {
    ros_env::sensor_msgs::msg::Joy {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        axes: bus.axes,
        buttons: crate::mappers::convert::FromI32Seq::from_i32_seq(bus.buttons),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJoyMapper;

impl TypedTopicMapper for SensorMsgsJoyMapper {
    type Ros = ros_env::sensor_msgs::msg::Joy;
    type Bus = robot_bus::sensor_msgs::msg::v1::Joy;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(joy_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(joy_to_ros(msg))
    }
}
