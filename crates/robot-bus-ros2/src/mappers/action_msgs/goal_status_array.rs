//! Typed mapper for `action_msgs/msg/GoalStatusArray`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn goal_status_array_to_bus(
    msg: ros_env::action_msgs::msg::GoalStatusArray,
) -> robot_bus::action_msgs::msg::v1::GoalStatusArray {
    robot_bus::action_msgs::msg::v1::GoalStatusArray {
        status_list: msg
            .status_list
            .into_iter()
            .map(crate::mappers::action_msgs::goal_status::goal_status_to_bus)
            .collect(),
    }
}

pub(crate) fn goal_status_array_to_ros(
    bus: robot_bus::action_msgs::msg::v1::GoalStatusArray,
) -> ros_env::action_msgs::msg::GoalStatusArray {
    ros_env::action_msgs::msg::GoalStatusArray {
        status_list: bus
            .status_list
            .into_iter()
            .map(crate::mappers::action_msgs::goal_status::goal_status_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ActionMsgsGoalStatusArrayMapper;

impl TypedTopicMapper for ActionMsgsGoalStatusArrayMapper {
    type Ros = ros_env::action_msgs::msg::GoalStatusArray;
    type Bus = robot_bus::action_msgs::msg::v1::GoalStatusArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(goal_status_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(goal_status_array_to_ros(msg))
    }
}
