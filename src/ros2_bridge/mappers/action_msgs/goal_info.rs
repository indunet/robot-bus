//! Mapper for `action_msgs/msg/GoalInfo`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn goal_info_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::action_msgs::msg::v1::GoalInfo> {
    Ok(crate::action_msgs::msg::v1::GoalInfo {
        goal_id: nested_view(view, "goal_id")?
            .as_ref()
            .map(super::super::unique_identifier_msgs::uuid::uuid_from_view)
            .transpose()?,
        stamp: nested_view(view, "stamp")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
    })
}

pub(crate) fn goal_info_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::action_msgs::msg::v1::GoalInfo,
) -> Result<()> {
    if let Some(v) = &bus.goal_id {
        with_nested_mut(view, "goal_id", |nested| {
            super::super::unique_identifier_msgs::uuid::uuid_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.stamp {
        with_nested_mut(view, "stamp", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn goal_info_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::action_msgs::msg::v1::GoalInfo> {
    goal_info_from_view(&msg.view())
}

pub(crate) fn goal_info_bus_to_dyn(
    bus: &crate::action_msgs::msg::v1::GoalInfo,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("action_msgs/msg/GoalInfo")?;
    goal_info_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ActionMsgsGoalInfoMapper;
impl TopicMapper for ActionMsgsGoalInfoMapper {
    fn type_name(&self) -> &'static str {
        "action_msgs/msg/GoalInfo"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(goal_info_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::action_msgs::msg::v1::GoalInfo as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode action_msgs/msg/GoalInfo: {e}")))?;
        goal_info_bus_to_dyn(&bus)
    }
}
