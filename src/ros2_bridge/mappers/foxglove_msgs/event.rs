//! Typed mapper for `foxglove_msgs/msg/Event`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn event_to_bus(msg: ros_env::foxglove_msgs::msg::Event) -> crate::foxglove_msgs::msg::v1::Event {
    crate::foxglove_msgs::msg::v1::Event {
        start_time: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.start_time)),
        end_time: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.end_time)),
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn event_to_ros(bus: crate::foxglove_msgs::msg::v1::Event) -> ros_env::foxglove_msgs::msg::Event {
    ros_env::foxglove_msgs::msg::Event {
        start_time: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.start_time.unwrap_or_default()),
        end_time: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.end_time.unwrap_or_default()),
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsEventMapper;

impl TypedTopicMapper for FoxgloveMsgsEventMapper {
    type Ros = ros_env::foxglove_msgs::msg::Event;
    type Bus = crate::foxglove_msgs::msg::v1::Event;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(event_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(event_to_ros(msg))
    }
}
