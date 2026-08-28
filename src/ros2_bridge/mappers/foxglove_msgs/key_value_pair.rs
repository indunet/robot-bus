//! Typed mapper for `foxglove_msgs/msg/KeyValuePair`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn key_value_pair_to_bus(msg: ros_env::foxglove_msgs::msg::KeyValuePair) -> crate::foxglove_msgs::msg::v1::KeyValuePair {
    crate::foxglove_msgs::msg::v1::KeyValuePair {
        key: crate::ros2_bridge::mappers::convert::from_ros_string(msg.key),
        value: crate::ros2_bridge::mappers::convert::from_ros_string(msg.value),
    }
}

pub(crate) fn key_value_pair_to_ros(bus: crate::foxglove_msgs::msg::v1::KeyValuePair) -> ros_env::foxglove_msgs::msg::KeyValuePair {
    ros_env::foxglove_msgs::msg::KeyValuePair {
        key: crate::ros2_bridge::mappers::convert::to_ros_string(bus.key),
        value: crate::ros2_bridge::mappers::convert::to_ros_string(bus.value),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsKeyValuePairMapper;

impl TypedTopicMapper for FoxgloveMsgsKeyValuePairMapper {
    type Ros = ros_env::foxglove_msgs::msg::KeyValuePair;
    type Bus = crate::foxglove_msgs::msg::v1::KeyValuePair;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/KeyValuePair"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(key_value_pair_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(key_value_pair_to_ros(msg))
    }
}
