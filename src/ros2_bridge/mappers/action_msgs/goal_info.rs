//! Typed mapper for `action_msgs/msg/GoalInfo`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn goal_info_to_bus(msg: ros_env::action_msgs::msg::GoalInfo) -> crate::action_msgs::msg::v1::GoalInfo {
    crate::action_msgs::msg::v1::GoalInfo {
        goal_id: Some(crate::ros2_bridge::mappers::unique_identifier_msgs::uuid::uuid_to_bus(msg.goal_id)),
        stamp: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.stamp)),
    }
}

pub(crate) fn goal_info_to_ros(bus: crate::action_msgs::msg::v1::GoalInfo) -> ros_env::action_msgs::msg::GoalInfo {
    ros_env::action_msgs::msg::GoalInfo {
        goal_id: crate::ros2_bridge::mappers::unique_identifier_msgs::uuid::uuid_to_ros(bus.goal_id.unwrap_or_default()),
        stamp: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.stamp.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ActionMsgsGoalInfoMapper;

impl TypedTopicMapper for ActionMsgsGoalInfoMapper {
    type Ros = ros_env::action_msgs::msg::GoalInfo;
    type Bus = crate::action_msgs::msg::v1::GoalInfo;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(goal_info_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(goal_info_to_ros(msg))
    }
}
