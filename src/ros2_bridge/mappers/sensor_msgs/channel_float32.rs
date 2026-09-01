//! Typed mapper for `sensor_msgs/msg/ChannelFloat32`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn channel_float32_to_bus(msg: ros_env::sensor_msgs::msg::ChannelFloat32) -> crate::sensor_msgs::msg::v1::ChannelFloat32 {
    crate::sensor_msgs::msg::v1::ChannelFloat32 {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        values: crate::ros2_bridge::mappers::convert::f32_seq(msg.values),
    }
}

pub(crate) fn channel_float32_to_ros(bus: crate::sensor_msgs::msg::v1::ChannelFloat32) -> ros_env::sensor_msgs::msg::ChannelFloat32 {
    ros_env::sensor_msgs::msg::ChannelFloat32 {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        values: bus.values,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsChannelFloat32Mapper;

impl TypedTopicMapper for SensorMsgsChannelFloat32Mapper {
    type Ros = ros_env::sensor_msgs::msg::ChannelFloat32;
    type Bus = crate::sensor_msgs::msg::v1::ChannelFloat32;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(channel_float32_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(channel_float32_to_ros(msg))
    }
}
