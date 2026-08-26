//! Mapper for `geometry_msgs/msg/WrenchStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn wrench_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::WrenchStamped> {
    Ok(crate::geometry_msgs::msg::v1::WrenchStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        wrench: nested_view(view, "wrench")?
            .as_ref()
            .map(super::wrench::wrench_from_view)
            .transpose()?,
    })
}

pub(crate) fn wrench_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::WrenchStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.wrench {
        with_nested_mut(view, "wrench", |nested| {
            super::wrench::wrench_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn wrench_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::WrenchStamped> {
    wrench_stamped_from_view(&msg.view())
}

pub(crate) fn wrench_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::WrenchStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/WrenchStamped")?;
    wrench_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsWrenchStampedMapper;
impl TopicMapper for GeometryMsgsWrenchStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/WrenchStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(wrench_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::WrenchStamped as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode geometry_msgs/msg/WrenchStamped: {e}"))
        })?;
        wrench_stamped_bus_to_dyn(&bus)
    }
}
