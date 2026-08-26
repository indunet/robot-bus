//! Mapper for `nav_msgs/msg/TrajectoryPoint`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn trajectory_point_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::TrajectoryPoint> {
    Ok(crate::nav_msgs::msg::v1::TrajectoryPoint {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
        velocity: nested_view(view, "velocity")?
            .as_ref()
            .map(super::super::geometry_msgs::twist::twist_from_view)
            .transpose()?,
        acceleration: nested_view(view, "acceleration")?
            .as_ref()
            .map(super::super::geometry_msgs::accel::accel_from_view)
            .transpose()?,
        effort: nested_view(view, "effort")?
            .as_ref()
            .map(super::super::geometry_msgs::wrench::wrench_from_view)
            .transpose()?,
    })
}

pub(crate) fn trajectory_point_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::TrajectoryPoint,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.velocity {
        with_nested_mut(view, "velocity", |nested| {
            super::super::geometry_msgs::twist::twist_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.acceleration {
        with_nested_mut(view, "acceleration", |nested| {
            super::super::geometry_msgs::accel::accel_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.effort {
        with_nested_mut(view, "effort", |nested| {
            super::super::geometry_msgs::wrench::wrench_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn trajectory_point_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::TrajectoryPoint> {
    trajectory_point_from_view(&msg.view())
}

pub(crate) fn trajectory_point_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::TrajectoryPoint,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/TrajectoryPoint")?;
    trajectory_point_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsTrajectoryPointMapper;
impl TopicMapper for NavMsgsTrajectoryPointMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/TrajectoryPoint"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(trajectory_point_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::TrajectoryPoint as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/TrajectoryPoint: {e}")))?;
        trajectory_point_bus_to_dyn(&bus)
    }
}
