//! Mapper for `nav2_msgs/msg/MissedWaypoint`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn missed_waypoint_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::MissedWaypoint> {
    Ok(crate::nav2_msgs::msg::v1::MissedWaypoint {
        index: read_u32(view, "index")?,
        goal: nested_view(view, "goal")?
            .as_ref()
            .map(super::super::geometry_msgs::pose_stamped::pose_stamped_from_view)
            .transpose()?,
        error_code: read_u32(view, "error_code")?,
    })
}

pub(crate) fn missed_waypoint_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::MissedWaypoint,
) -> Result<()> {
    write_u32(view, "index", bus.index)?;
    if let Some(v) = &bus.goal {
        with_nested_mut(view, "goal", |nested| {
            super::super::geometry_msgs::pose_stamped::pose_stamped_write(nested, v)
        })?;
    }
    write_u32(view, "error_code", bus.error_code)?;
    Ok(())
}

pub(crate) fn missed_waypoint_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::MissedWaypoint> {
    missed_waypoint_from_view(&msg.view())
}

pub(crate) fn missed_waypoint_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::MissedWaypoint,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/MissedWaypoint")?;
    missed_waypoint_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsMissedWaypointMapper;
impl TopicMapper for Nav2MsgsMissedWaypointMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/MissedWaypoint"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(missed_waypoint_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::MissedWaypoint as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/MissedWaypoint: {e}")))?;
        missed_waypoint_bus_to_dyn(&bus)
    }
}
