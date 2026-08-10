//! Mapper for `nav2_msgs/msg/BehaviorTreeLog`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn behavior_tree_log_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::BehaviorTreeLog> {
    Ok(crate::nav2_msgs::msg::v1::BehaviorTreeLog {
        timestamp: nested_view(view, "timestamp")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        event_log: read_message_seq(view, "event_log", super::behavior_tree_status_change::behavior_tree_status_change_from_view)?,
    })
}

pub(crate) fn behavior_tree_log_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::BehaviorTreeLog,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        with_nested_mut(view, "timestamp", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "event_log",
        &bus.event_log,
        super::behavior_tree_status_change::behavior_tree_status_change_write,
    )?;
    Ok(())
}

pub(crate) fn behavior_tree_log_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::BehaviorTreeLog> {
    behavior_tree_log_from_view(&msg.view())
}

pub(crate) fn behavior_tree_log_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::BehaviorTreeLog,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/BehaviorTreeLog")?;
    behavior_tree_log_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsBehaviorTreeLogMapper;
impl TopicMapper for Nav2MsgsBehaviorTreeLogMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/BehaviorTreeLog"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(behavior_tree_log_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::BehaviorTreeLog as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode nav2_msgs/msg/BehaviorTreeLog: {e}"))
            })?;
        behavior_tree_log_bus_to_dyn(&bus)
    }
}
