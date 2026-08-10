//! Mapper for `geometry_msgs/msg/Inertia`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn inertia_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Inertia> {
    Ok(crate::geometry_msgs::msg::v1::Inertia {
        m: read_f64(view, "m")?,
        com: nested_view(view, "com")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        ixx: read_f64(view, "ixx")?,
        ixy: read_f64(view, "ixy")?,
        ixz: read_f64(view, "ixz")?,
        iyy: read_f64(view, "iyy")?,
        iyz: read_f64(view, "iyz")?,
        izz: read_f64(view, "izz")?,
    })
}

pub(crate) fn inertia_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Inertia,
) -> Result<()> {
    write_f64(view, "m", bus.m)?;
    if let Some(v) = &bus.com {
        with_nested_mut(view, "com", |nested| super::vector3::vector3_write(nested, v))?;
    }
    write_f64(view, "ixx", bus.ixx)?;
    write_f64(view, "ixy", bus.ixy)?;
    write_f64(view, "ixz", bus.ixz)?;
    write_f64(view, "iyy", bus.iyy)?;
    write_f64(view, "iyz", bus.iyz)?;
    write_f64(view, "izz", bus.izz)?;
    Ok(())
}

pub(crate) fn inertia_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Inertia> {
    inertia_from_view(&msg.view())
}

pub(crate) fn inertia_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Inertia,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Inertia")?;
    inertia_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsInertiaMapper;
impl TopicMapper for GeometryMsgsInertiaMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Inertia"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(inertia_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Inertia as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Inertia: {e}")))?;
        inertia_bus_to_dyn(&bus)
    }
}
