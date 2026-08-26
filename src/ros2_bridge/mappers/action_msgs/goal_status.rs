//! Mapper for `action_msgs/msg/GoalStatus`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn goal_status_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::action_msgs::msg::v1::GoalStatus> {
    Ok(crate::action_msgs::msg::v1::GoalStatus {
        goal_info: nested_view(view, "goal_info")?
            .as_ref()
            .map(super::goal_info::goal_info_from_view)
            .transpose()?,
        status: read_i32(view, "status")?,
    })
}

pub(crate) fn goal_status_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::action_msgs::msg::v1::GoalStatus,
) -> Result<()> {
    if let Some(v) = &bus.goal_info {
        with_nested_mut(view, "goal_info", |nested| {
            super::goal_info::goal_info_write(nested, v)
        })?;
    }
    write_i32(view, "status", bus.status)?;
    Ok(())
}

pub(crate) fn goal_status_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::action_msgs::msg::v1::GoalStatus> {
    goal_status_from_view(&msg.view())
}

pub(crate) fn goal_status_bus_to_dyn(
    bus: &crate::action_msgs::msg::v1::GoalStatus,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("action_msgs/msg/GoalStatus")?;
    goal_status_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ActionMsgsGoalStatusMapper;
impl TopicMapper for ActionMsgsGoalStatusMapper {
    fn type_name(&self) -> &'static str {
        "action_msgs/msg/GoalStatus"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(goal_status_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::action_msgs::msg::v1::GoalStatus as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode action_msgs/msg/GoalStatus: {e}")))?;
        goal_status_bus_to_dyn(&bus)
    }
}
