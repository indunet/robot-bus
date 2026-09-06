//! Typed mapper for `action_msgs/msg/GoalStatusArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn goal_status_array_to_bus(
    msg: ros_env::action_msgs::msg::GoalStatusArray,
) -> crate::action_msgs::msg::v1::GoalStatusArray {
    crate::action_msgs::msg::v1::GoalStatusArray {
        status_list: msg
            .status_list
            .into_iter()
            .map(crate::ros2_bridge::mappers::action_msgs::goal_status::goal_status_to_bus)
            .collect(),
    }
}

pub(crate) fn goal_status_array_to_ros(
    bus: crate::action_msgs::msg::v1::GoalStatusArray,
) -> ros_env::action_msgs::msg::GoalStatusArray {
    ros_env::action_msgs::msg::GoalStatusArray {
        status_list: bus
            .status_list
            .into_iter()
            .map(crate::ros2_bridge::mappers::action_msgs::goal_status::goal_status_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ActionMsgsGoalStatusArrayMapper;

impl TypedTopicMapper for ActionMsgsGoalStatusArrayMapper {
    type Ros = ros_env::action_msgs::msg::GoalStatusArray;
    type Bus = crate::action_msgs::msg::v1::GoalStatusArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(goal_status_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(goal_status_array_to_ros(msg))
    }
}
