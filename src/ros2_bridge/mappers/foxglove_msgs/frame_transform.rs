//! Mapper for `foxglove_msgs/msg/FrameTransform`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn frame_transform_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::FrameTransform> {
    Ok(crate::foxglove_msgs::msg::v1::FrameTransform {
        timestamp: read_timestamp(view, "timestamp")?,
        parent_frame_id: read_string(view, "parent_frame_id")?,
        child_frame_id: read_string(view, "child_frame_id")?,
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

pub(crate) fn frame_transform_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::FrameTransform,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "parent_frame_id", &bus.parent_frame_id)?;
    write_string(view, "child_frame_id", &bus.child_frame_id)?;
    if let Some(v) = &bus.translation {
        with_nested_mut(view, "translation", |nested| super::vector3::vector3_write(nested, v))?;
    }
    if let Some(v) = &bus.rotation {
        with_nested_mut(view, "rotation", |nested| super::quaternion::quaternion_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn frame_transform_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::FrameTransform> {
    frame_transform_from_view(&msg.view())
}

pub(crate) fn frame_transform_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::FrameTransform,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/FrameTransform")?;
    frame_transform_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsFrameTransformMapper;
impl TopicMapper for FoxgloveMsgsFrameTransformMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/FrameTransform"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(frame_transform_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::FrameTransform as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/FrameTransform: {e}"))
            })?;
        frame_transform_bus_to_dyn(&bus)
    }
}
