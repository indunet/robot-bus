//! Mapper for `geometry_msgs/msg/Twist`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn twist_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Twist> {
    Ok(crate::geometry_msgs::msg::v1::Twist {
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

pub(crate) fn twist_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Twist,
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

pub(crate) fn twist_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Twist> {
    twist_from_view(&msg.view())
}

pub(crate) fn twist_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Twist,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Twist")?;
    twist_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsTwistMapper;
impl TopicMapper for GeometryMsgsTwistMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Twist"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(twist_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Twist as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Twist: {e}")))?;
        twist_bus_to_dyn(&bus)
    }
}
