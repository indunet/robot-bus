//! Mapper for `geometry_msgs/msg/PointStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::PointStamped> {
    Ok(crate::geometry_msgs::msg::v1::PointStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        point: nested_view(view, "point")?
            .as_ref()
            .map(super::point::point_from_view)
            .transpose()?,
    })
}

pub(crate) fn point_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::PointStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.point {
        with_nested_mut(view, "point", |nested| super::point::point_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn point_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::PointStamped> {
    point_stamped_from_view(&msg.view())
}

pub(crate) fn point_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::PointStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/PointStamped")?;
    point_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPointStampedMapper;
impl TopicMapper for GeometryMsgsPointStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PointStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::PointStamped as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode geometry_msgs/msg/PointStamped: {e}"))
            })?;
        point_stamped_bus_to_dyn(&bus)
    }
}
