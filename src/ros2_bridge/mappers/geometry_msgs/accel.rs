//! Mapper for `geometry_msgs/msg/Accel`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn accel_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Accel> {
    Ok(crate::geometry_msgs::msg::v1::Accel {
        linear: nested_view(view, "linear")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        angular: nested_view(view, "angular")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
    })
}

pub(crate) fn accel_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Accel,
) -> Result<()> {
    if let Some(v) = &bus.linear {
        with_nested_mut(view, "linear", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.angular {
        with_nested_mut(view, "angular", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn accel_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Accel> {
    accel_from_view(&msg.view())
}

pub(crate) fn accel_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Accel,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Accel")?;
    accel_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsAccelMapper;
impl TopicMapper for GeometryMsgsAccelMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Accel"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(accel_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Accel as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Accel: {e}")))?;
        accel_bus_to_dyn(&bus)
    }
}
