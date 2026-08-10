//! Mapper for `nav2_msgs/msg/CollisionMonitorState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn collision_monitor_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::CollisionMonitorState> {
    Ok(crate::nav2_msgs::msg::v1::CollisionMonitorState {
        action_type: read_u32(view, "action_type")?,
        polygon_name: read_string(view, "polygon_name")?,
    })
}

pub(crate) fn collision_monitor_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::CollisionMonitorState,
) -> Result<()> {
    write_u32(view, "action_type", bus.action_type)?;
    write_string(view, "polygon_name", &bus.polygon_name)?;
    Ok(())
}

pub(crate) fn collision_monitor_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::CollisionMonitorState> {
    collision_monitor_state_from_view(&msg.view())
}

pub(crate) fn collision_monitor_state_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::CollisionMonitorState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/CollisionMonitorState")?;
    collision_monitor_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsCollisionMonitorStateMapper;
impl TopicMapper for Nav2MsgsCollisionMonitorStateMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/CollisionMonitorState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(collision_monitor_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::nav2_msgs::msg::v1::CollisionMonitorState as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode nav2_msgs/msg/CollisionMonitorState: {e}"))
                })?;
        collision_monitor_state_bus_to_dyn(&bus)
    }
}
