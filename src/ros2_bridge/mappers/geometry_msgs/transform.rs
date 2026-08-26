//! Mapper for `geometry_msgs/msg/Transform`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn transform_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Transform> {
    Ok(crate::geometry_msgs::msg::v1::Transform {
        translation: nested_view(view, "translation")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        rotation: nested_view(view, "rotation")?
            .as_ref()
            .map(super::quaternion::quaternion_from_view)
            .transpose()?,
    })
}

pub(crate) fn transform_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Transform,
) -> Result<()> {
    if let Some(v) = &bus.translation {
        with_nested_mut(view, "translation", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.rotation {
        with_nested_mut(view, "rotation", |nested| {
            super::quaternion::quaternion_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn transform_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Transform> {
    transform_from_view(&msg.view())
}

pub(crate) fn transform_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Transform,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Transform")?;
    transform_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsTransformMapper;
impl TopicMapper for GeometryMsgsTransformMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Transform"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(transform_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Transform as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Transform: {e}")))?;
        transform_bus_to_dyn(&bus)
    }
}
