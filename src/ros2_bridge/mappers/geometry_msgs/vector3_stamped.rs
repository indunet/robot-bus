//! Mapper for `geometry_msgs/msg/Vector3Stamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn vector3_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Vector3Stamped> {
    Ok(crate::geometry_msgs::msg::v1::Vector3Stamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        vector: nested_view(view, "vector")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
    })
}

pub(crate) fn vector3_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Vector3Stamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.vector {
        with_nested_mut(view, "vector", |nested| super::vector3::vector3_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn vector3_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Vector3Stamped> {
    vector3_stamped_from_view(&msg.view())
}

pub(crate) fn vector3_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Vector3Stamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Vector3Stamped")?;
    vector3_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsVector3StampedMapper;
impl TopicMapper for GeometryMsgsVector3StampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Vector3Stamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(vector3_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Vector3Stamped as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode geometry_msgs/msg/Vector3Stamped: {e}"))
            })?;
        vector3_stamped_bus_to_dyn(&bus)
    }
}
