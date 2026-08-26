//! Mapper for `geometry_msgs/msg/VelocityWithCovarianceStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn velocity_with_covariance_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped> {
    Ok(
        crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped {
            header: nested_view(view, "header")?
                .as_ref()
                .map(super::super::std_msgs::header::header_from_view)
                .transpose()?,
            body_frame_id: read_string(view, "body_frame_id")?,
            reference_frame_id: read_string(view, "reference_frame_id")?,
            velocity: nested_view(view, "velocity")?
                .as_ref()
                .map(super::twist_with_covariance::twist_with_covariance_from_view)
                .transpose()?,
        },
    )
}

pub(crate) fn velocity_with_covariance_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string(view, "body_frame_id", &bus.body_frame_id)?;
    write_string(view, "reference_frame_id", &bus.reference_frame_id)?;
    if let Some(v) = &bus.velocity {
        with_nested_mut(view, "velocity", |nested| {
            super::twist_with_covariance::twist_with_covariance_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn velocity_with_covariance_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped> {
    velocity_with_covariance_stamped_from_view(&msg.view())
}

pub(crate) fn velocity_with_covariance_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/VelocityWithCovarianceStamped")?;
    velocity_with_covariance_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsVelocityWithCovarianceStampedMapper;
impl TopicMapper for GeometryMsgsVelocityWithCovarianceStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/VelocityWithCovarianceStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(velocity_with_covariance_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode geometry_msgs/msg/VelocityWithCovarianceStamped: {e}"
                ))
            })?;
        velocity_with_covariance_stamped_bus_to_dyn(&bus)
    }
}
