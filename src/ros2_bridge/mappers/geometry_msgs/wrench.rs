//! Mapper for `geometry_msgs/msg/Wrench`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn wrench_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Wrench> {
    Ok(crate::geometry_msgs::msg::v1::Wrench {
        force: nested_view(view, "force")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        torque: nested_view(view, "torque")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
    })
}

pub(crate) fn wrench_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Wrench,
) -> Result<()> {
    if let Some(v) = &bus.force {
        with_nested_mut(view, "force", |nested| super::vector3::vector3_write(nested, v))?;
    }
    if let Some(v) = &bus.torque {
        with_nested_mut(view, "torque", |nested| super::vector3::vector3_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn wrench_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Wrench> {
    wrench_from_view(&msg.view())
}

pub(crate) fn wrench_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Wrench,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Wrench")?;
    wrench_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsWrenchMapper;
impl TopicMapper for GeometryMsgsWrenchMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Wrench"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(wrench_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Wrench as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Wrench: {e}")))?;
        wrench_bus_to_dyn(&bus)
    }
}
