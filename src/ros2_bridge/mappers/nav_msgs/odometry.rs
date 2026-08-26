//! Mapper for `nav_msgs/msg/Odometry`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn odometry_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::Odometry> {
    Ok(crate::nav_msgs::msg::v1::Odometry {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        child_frame_id: read_string(view, "child_frame_id")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::super::geometry_msgs::pose_with_covariance::pose_with_covariance_from_view)
            .transpose()?,
        twist: nested_view(view, "twist")?
            .as_ref()
            .map(
                super::super::geometry_msgs::twist_with_covariance::twist_with_covariance_from_view,
            )
            .transpose()?,
    })
}

pub(crate) fn odometry_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::Odometry,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string(view, "child_frame_id", &bus.child_frame_id)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose_with_covariance::pose_with_covariance_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.twist {
        with_nested_mut(view, "twist", |nested| {
            super::super::geometry_msgs::twist_with_covariance::twist_with_covariance_write(
                nested, v,
            )
        })?;
    }
    Ok(())
}

pub(crate) fn odometry_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::Odometry> {
    odometry_from_view(&msg.view())
}

pub(crate) fn odometry_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::Odometry,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/Odometry")?;
    odometry_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsOdometryMapper;
impl TopicMapper for NavMsgsOdometryMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/Odometry"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(odometry_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::Odometry as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/Odometry: {e}")))?;
        odometry_bus_to_dyn(&bus)
    }
}
