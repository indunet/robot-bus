//! Mapper for `geometry_msgs/msg/InertiaStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn inertia_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::InertiaStamped> {
    Ok(crate::geometry_msgs::msg::v1::InertiaStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        inertia: nested_view(view, "inertia")?
            .as_ref()
            .map(super::inertia::inertia_from_view)
            .transpose()?,
    })
}

pub(crate) fn inertia_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::InertiaStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.inertia {
        with_nested_mut(view, "inertia", |nested| super::inertia::inertia_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn inertia_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::InertiaStamped> {
    inertia_stamped_from_view(&msg.view())
}

pub(crate) fn inertia_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::InertiaStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/InertiaStamped")?;
    inertia_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsInertiaStampedMapper;
impl TopicMapper for GeometryMsgsInertiaStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/InertiaStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(inertia_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::InertiaStamped as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode geometry_msgs/msg/InertiaStamped: {e}"))
            })?;
        inertia_stamped_bus_to_dyn(&bus)
    }
}
