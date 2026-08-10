//! Mapper for `nav_msgs/msg/Goals`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn goals_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::Goals> {
    Ok(crate::nav_msgs::msg::v1::Goals {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        goals: read_message_seq(view, "goals", super::super::geometry_msgs::pose_stamped::pose_stamped_from_view)?,
    })
}

pub(crate) fn goals_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::Goals,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "goals",
        &bus.goals,
        super::super::geometry_msgs::pose_stamped::pose_stamped_write,
    )?;
    Ok(())
}

pub(crate) fn goals_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::Goals> {
    goals_from_view(&msg.view())
}

pub(crate) fn goals_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::Goals,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/Goals")?;
    goals_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsGoalsMapper;
impl TopicMapper for NavMsgsGoalsMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/Goals"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(goals_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::Goals as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/Goals: {e}")))?;
        goals_bus_to_dyn(&bus)
    }
}
