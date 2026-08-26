//! Mapper for `foxglove_msgs/msg/Odometry`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn odometry_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Odometry> {
    Ok(crate::foxglove_msgs::msg::v1::Odometry {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        body_frame_id: read_string(view, "body_frame_id")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        linear_velocity: nested_view(view, "linear_velocity")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        angular_velocity: nested_view(view, "angular_velocity")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        pose_covariance: read_f64_seq(view, "pose_covariance")?,
        velocity_covariance: read_f64_seq(view, "velocity_covariance")?,
        metadata: read_message_seq(
            view,
            "metadata",
            super::key_value_pair::key_value_pair_from_view,
        )?,
    })
}

pub(crate) fn odometry_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Odometry,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_string(view, "body_frame_id", &bus.body_frame_id)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    if let Some(v) = &bus.linear_velocity {
        with_nested_mut(view, "linear_velocity", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.angular_velocity {
        with_nested_mut(view, "angular_velocity", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    write_f64_seq(view, "pose_covariance", &bus.pose_covariance)?;
    write_f64_seq(view, "velocity_covariance", &bus.velocity_covariance)?;
    write_message_seq(
        view,
        "metadata",
        &bus.metadata,
        super::key_value_pair::key_value_pair_write,
    )?;
    Ok(())
}

pub(crate) fn odometry_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Odometry> {
    odometry_from_view(&msg.view())
}

pub(crate) fn odometry_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Odometry,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Odometry")?;
    odometry_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsOdometryMapper;
impl TopicMapper for FoxgloveMsgsOdometryMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Odometry"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(odometry_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Odometry as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Odometry: {e}")))?;
        odometry_bus_to_dyn(&bus)
    }
}
