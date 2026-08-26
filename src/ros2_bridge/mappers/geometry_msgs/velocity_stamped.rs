//! Mapper for `geometry_msgs/msg/VelocityStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn velocity_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::VelocityStamped> {
    Ok(crate::geometry_msgs::msg::v1::VelocityStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        body_frame_id: read_string(view, "body_frame_id")?,
        reference_frame_id: read_string(view, "reference_frame_id")?,
        velocity: nested_view(view, "velocity")?
            .as_ref()
            .map(super::twist::twist_from_view)
            .transpose()?,
    })
}

pub(crate) fn velocity_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::VelocityStamped,
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
            super::twist::twist_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn velocity_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::VelocityStamped> {
    velocity_stamped_from_view(&msg.view())
}

pub(crate) fn velocity_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::VelocityStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/VelocityStamped")?;
    velocity_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsVelocityStampedMapper;
impl TopicMapper for GeometryMsgsVelocityStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/VelocityStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(velocity_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::VelocityStamped as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode geometry_msgs/msg/VelocityStamped: {e}"))
            })?;
        velocity_stamped_bus_to_dyn(&bus)
    }
}
