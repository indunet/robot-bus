//! Typed mapper for `diagnostic_msgs/msg/KeyValue`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn key_value_to_bus(msg: ros_env::diagnostic_msgs::msg::KeyValue) -> crate::diagnostic_msgs::msg::v1::KeyValue {
    crate::diagnostic_msgs::msg::v1::KeyValue {
        key: crate::ros2_bridge::mappers::convert::from_ros_string(msg.key),
        value: crate::ros2_bridge::mappers::convert::from_ros_string(msg.value),
    }
}

pub(crate) fn key_value_to_ros(bus: crate::diagnostic_msgs::msg::v1::KeyValue) -> ros_env::diagnostic_msgs::msg::KeyValue {
    ros_env::diagnostic_msgs::msg::KeyValue {
        key: crate::ros2_bridge::mappers::convert::to_ros_string(bus.key),
        value: crate::ros2_bridge::mappers::convert::to_ros_string(bus.value),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DiagnosticMsgsKeyValueMapper;

impl TypedTopicMapper for DiagnosticMsgsKeyValueMapper {
    type Ros = ros_env::diagnostic_msgs::msg::KeyValue;
    type Bus = crate::diagnostic_msgs::msg::v1::KeyValue;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(key_value_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(key_value_to_ros(msg))
    }
}
