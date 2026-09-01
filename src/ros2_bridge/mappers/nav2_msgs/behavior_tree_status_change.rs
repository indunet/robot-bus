//! Typed mapper for `nav2_msgs/msg/BehaviorTreeStatusChange`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn behavior_tree_status_change_to_bus(msg: ros_env::nav2_msgs::msg::BehaviorTreeStatusChange) -> crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange {
    crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange {
        timestamp: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.timestamp)),
        node_name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.node_name),
        previous_status: crate::ros2_bridge::mappers::convert::from_ros_string(msg.previous_status),
        current_status: crate::ros2_bridge::mappers::convert::from_ros_string(msg.current_status),
    }
}

pub(crate) fn behavior_tree_status_change_to_ros(bus: crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange) -> ros_env::nav2_msgs::msg::BehaviorTreeStatusChange {
    ros_env::nav2_msgs::msg::BehaviorTreeStatusChange {
        timestamp: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.timestamp.unwrap_or_default()),
        node_name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.node_name),
        previous_status: crate::ros2_bridge::mappers::convert::to_ros_string(bus.previous_status),
        current_status: crate::ros2_bridge::mappers::convert::to_ros_string(bus.current_status),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsBehaviorTreeStatusChangeMapper;

impl TypedTopicMapper for Nav2MsgsBehaviorTreeStatusChangeMapper {
    type Ros = ros_env::nav2_msgs::msg::BehaviorTreeStatusChange;
    type Bus = crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(behavior_tree_status_change_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(behavior_tree_status_change_to_ros(msg))
    }
}
