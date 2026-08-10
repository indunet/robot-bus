//! Mapper for `nav2_msgs/msg/BehaviorTreeStatusChange`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn behavior_tree_status_change_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange> {
    Ok(crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange {
        timestamp: nested_view(view, "timestamp")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        node_name: read_string(view, "node_name")?,
        previous_status: read_string(view, "previous_status")?,
        current_status: read_string(view, "current_status")?,
    })
}

pub(crate) fn behavior_tree_status_change_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        with_nested_mut(view, "timestamp", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    write_string(view, "node_name", &bus.node_name)?;
    write_string(view, "previous_status", &bus.previous_status)?;
    write_string(view, "current_status", &bus.current_status)?;
    Ok(())
}

pub(crate) fn behavior_tree_status_change_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange> {
    behavior_tree_status_change_from_view(&msg.view())
}

pub(crate) fn behavior_tree_status_change_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/BehaviorTreeStatusChange")?;
    behavior_tree_status_change_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsBehaviorTreeStatusChangeMapper;
impl TopicMapper for Nav2MsgsBehaviorTreeStatusChangeMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/BehaviorTreeStatusChange"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(behavior_tree_status_change_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::nav2_msgs::msg::v1::BehaviorTreeStatusChange as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!(
                        "decode nav2_msgs/msg/BehaviorTreeStatusChange: {e}"
                    ))
                })?;
        behavior_tree_status_change_bus_to_dyn(&bus)
    }
}
