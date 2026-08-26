//! Mapper for `foxglove_msgs/msg/FrameTransforms`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn frame_transforms_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::FrameTransforms> {
    Ok(crate::foxglove_msgs::msg::v1::FrameTransforms {
        transforms: read_message_seq(
            view,
            "transforms",
            super::frame_transform::frame_transform_from_view,
        )?,
    })
}

pub(crate) fn frame_transforms_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::FrameTransforms,
) -> Result<()> {
    write_message_seq(
        view,
        "transforms",
        &bus.transforms,
        super::frame_transform::frame_transform_write,
    )?;
    Ok(())
}

pub(crate) fn frame_transforms_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::FrameTransforms> {
    frame_transforms_from_view(&msg.view())
}

pub(crate) fn frame_transforms_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::FrameTransforms,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/FrameTransforms")?;
    frame_transforms_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsFrameTransformsMapper;
impl TopicMapper for FoxgloveMsgsFrameTransformsMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/FrameTransforms"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(frame_transforms_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::FrameTransforms as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/FrameTransforms: {e}"))
            })?;
        frame_transforms_bus_to_dyn(&bus)
    }
}
