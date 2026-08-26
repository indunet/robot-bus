//! Mapper for `action_msgs/msg/GoalStatusArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn goal_status_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::action_msgs::msg::v1::GoalStatusArray> {
    Ok(crate::action_msgs::msg::v1::GoalStatusArray {
        status_list: read_message_seq(
            view,
            "status_list",
            super::goal_status::goal_status_from_view,
        )?,
    })
}

pub(crate) fn goal_status_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::action_msgs::msg::v1::GoalStatusArray,
) -> Result<()> {
    write_message_seq(
        view,
        "status_list",
        &bus.status_list,
        super::goal_status::goal_status_write,
    )?;
    Ok(())
}

pub(crate) fn goal_status_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::action_msgs::msg::v1::GoalStatusArray> {
    goal_status_array_from_view(&msg.view())
}

pub(crate) fn goal_status_array_bus_to_dyn(
    bus: &crate::action_msgs::msg::v1::GoalStatusArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("action_msgs/msg/GoalStatusArray")?;
    goal_status_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ActionMsgsGoalStatusArrayMapper;
impl TopicMapper for ActionMsgsGoalStatusArrayMapper {
    fn type_name(&self) -> &'static str {
        "action_msgs/msg/GoalStatusArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(goal_status_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::action_msgs::msg::v1::GoalStatusArray as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode action_msgs/msg/GoalStatusArray: {e}"))
        })?;
        goal_status_array_bus_to_dyn(&bus)
    }
}
