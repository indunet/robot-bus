//! Typed mapper for `foxglove_msgs/msg/Log`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn log_to_bus(msg: ros_env::foxglove_msgs::msg::Log) -> crate::foxglove_msgs::msg::v1::Log {
    crate::foxglove_msgs::msg::v1::Log {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        level: msg.level as i32,
        message: crate::ros2_bridge::mappers::convert::from_ros_string(msg.message),
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        file: crate::ros2_bridge::mappers::convert::from_ros_string(msg.file),
        line: msg.line,
    }
}

pub(crate) fn log_to_ros(bus: crate::foxglove_msgs::msg::v1::Log) -> ros_env::foxglove_msgs::msg::Log {
    ros_env::foxglove_msgs::msg::Log {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        level: bus.level as i32,
        message: crate::ros2_bridge::mappers::convert::to_ros_string(bus.message),
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        file: crate::ros2_bridge::mappers::convert::to_ros_string(bus.file),
        line: bus.line,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsLogMapper;

impl TypedTopicMapper for FoxgloveMsgsLogMapper {
    type Ros = ros_env::foxglove_msgs::msg::Log;
    type Bus = crate::foxglove_msgs::msg::v1::Log;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Log"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(log_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(log_to_ros(msg))
    }
}
