//! Typed mapper for `action_msgs/msg/GoalInfo`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn goal_info_to_bus(
    msg: ros_env::action_msgs::msg::GoalInfo,
) -> robot_bus::action_msgs::msg::v1::GoalInfo {
    robot_bus::action_msgs::msg::v1::GoalInfo {
        goal_id: Some(
            crate::mappers::unique_identifier_msgs::uuid::uuid_to_bus(msg.goal_id),
        ),
        stamp: Some(crate::mappers::builtin_interfaces::time::time_to_bus(msg.stamp)),
    }
}

pub(crate) fn goal_info_to_ros(
    bus: robot_bus::action_msgs::msg::v1::GoalInfo,
) -> ros_env::action_msgs::msg::GoalInfo {
    ros_env::action_msgs::msg::GoalInfo {
        goal_id: crate::mappers::unique_identifier_msgs::uuid::uuid_to_ros(
            bus.goal_id.unwrap_or_default(),
        ),
        stamp: crate::mappers::builtin_interfaces::time::time_to_ros(
            bus.stamp.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ActionMsgsGoalInfoMapper;

impl TypedTopicMapper for ActionMsgsGoalInfoMapper {
    type Ros = ros_env::action_msgs::msg::GoalInfo;
    type Bus = robot_bus::action_msgs::msg::v1::GoalInfo;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(goal_info_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(goal_info_to_ros(msg))
    }
}
