//! Mapper for `nav_msgs/msg/Trajectory`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn trajectory_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::Trajectory> {
    Ok(crate::nav_msgs::msg::v1::Trajectory {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        points: read_message_seq(
            view,
            "points",
            super::trajectory_point::trajectory_point_from_view,
        )?,
    })
}

pub(crate) fn trajectory_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::Trajectory,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "points",
        &bus.points,
        super::trajectory_point::trajectory_point_write,
    )?;
    Ok(())
}

pub(crate) fn trajectory_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::Trajectory> {
    trajectory_from_view(&msg.view())
}

pub(crate) fn trajectory_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::Trajectory,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/Trajectory")?;
    trajectory_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsTrajectoryMapper;
impl TopicMapper for NavMsgsTrajectoryMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/Trajectory"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(trajectory_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::Trajectory as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/Trajectory: {e}")))?;
        trajectory_bus_to_dyn(&bus)
    }
}
