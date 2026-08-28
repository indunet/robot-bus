//! Typed mapper for `nav2_msgs/msg/BehaviorTreeLog`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn behavior_tree_log_to_bus(msg: ros_env::nav2_msgs::msg::BehaviorTreeLog) -> crate::nav2_msgs::msg::v1::BehaviorTreeLog {
    crate::nav2_msgs::msg::v1::BehaviorTreeLog {
        timestamp: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.timestamp)),
        event_log: msg.event_log.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::behavior_tree_status_change::behavior_tree_status_change_to_bus).collect(),
    }
}

pub(crate) fn behavior_tree_log_to_ros(bus: crate::nav2_msgs::msg::v1::BehaviorTreeLog) -> ros_env::nav2_msgs::msg::BehaviorTreeLog {
    ros_env::nav2_msgs::msg::BehaviorTreeLog {
        timestamp: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.timestamp.unwrap_or_default()),
        event_log: bus.event_log.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::behavior_tree_status_change::behavior_tree_status_change_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsBehaviorTreeLogMapper;

impl TypedTopicMapper for Nav2MsgsBehaviorTreeLogMapper {
    type Ros = ros_env::nav2_msgs::msg::BehaviorTreeLog;
    type Bus = crate::nav2_msgs::msg::v1::BehaviorTreeLog;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/BehaviorTreeLog"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(behavior_tree_log_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(behavior_tree_log_to_ros(msg))
    }
}
