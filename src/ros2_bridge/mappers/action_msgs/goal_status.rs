//! Typed mapper for `action_msgs/msg/GoalStatus`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn goal_status_to_bus(msg: ros_env::action_msgs::msg::GoalStatus) -> crate::action_msgs::msg::v1::GoalStatus {
    crate::action_msgs::msg::v1::GoalStatus {
        goal_info: Some(crate::ros2_bridge::mappers::action_msgs::goal_info::goal_info_to_bus(msg.goal_info)),
        status: i32::from(msg.status),
    }
}

pub(crate) fn goal_status_to_ros(bus: crate::action_msgs::msg::v1::GoalStatus) -> ros_env::action_msgs::msg::GoalStatus {
    ros_env::action_msgs::msg::GoalStatus {
        goal_info: crate::ros2_bridge::mappers::action_msgs::goal_info::goal_info_to_ros(bus.goal_info.unwrap_or_default()),
        status: bus.status as i8,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ActionMsgsGoalStatusMapper;

impl TypedTopicMapper for ActionMsgsGoalStatusMapper {
    type Ros = ros_env::action_msgs::msg::GoalStatus;
    type Bus = crate::action_msgs::msg::v1::GoalStatus;

    fn type_name(&self) -> &'static str {
        "action_msgs/msg/GoalStatus"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(goal_status_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(goal_status_to_ros(msg))
    }
}
