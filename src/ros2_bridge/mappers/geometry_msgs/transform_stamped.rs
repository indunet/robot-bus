//! Mapper for `geometry_msgs/msg/TransformStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn transform_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::TransformStamped> {
    Ok(crate::geometry_msgs::msg::v1::TransformStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        child_frame_id: read_string(view, "child_frame_id")?,
        transform: nested_view(view, "transform")?
            .as_ref()
            .map(super::transform::transform_from_view)
            .transpose()?,
    })
}

pub(crate) fn transform_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::TransformStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string(view, "child_frame_id", &bus.child_frame_id)?;
    if let Some(v) = &bus.transform {
        with_nested_mut(view, "transform", |nested| {
            super::transform::transform_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn transform_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::TransformStamped> {
    transform_stamped_from_view(&msg.view())
}

pub(crate) fn transform_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::TransformStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/TransformStamped")?;
    transform_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsTransformStampedMapper;
impl TopicMapper for GeometryMsgsTransformStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/TransformStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(transform_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::geometry_msgs::msg::v1::TransformStamped as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode geometry_msgs/msg/TransformStamped: {e}"))
                })?;
        transform_stamped_bus_to_dyn(&bus)
    }
}
