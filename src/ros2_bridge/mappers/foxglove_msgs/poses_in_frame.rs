//! Mapper for `foxglove_msgs/msg/PosesInFrame`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn poses_in_frame_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::PosesInFrame> {
    Ok(crate::foxglove_msgs::msg::v1::PosesInFrame {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        poses: read_message_seq(view, "poses", super::pose::pose_from_view)?,
    })
}

pub(crate) fn poses_in_frame_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::PosesInFrame,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_message_seq(view, "poses", &bus.poses, super::pose::pose_write)?;
    Ok(())
}

pub(crate) fn poses_in_frame_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::PosesInFrame> {
    poses_in_frame_from_view(&msg.view())
}

pub(crate) fn poses_in_frame_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::PosesInFrame,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/PosesInFrame")?;
    poses_in_frame_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPosesInFrameMapper;
impl TopicMapper for FoxgloveMsgsPosesInFrameMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PosesInFrame"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(poses_in_frame_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::PosesInFrame as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/PosesInFrame: {e}"))
            })?;
        poses_in_frame_bus_to_dyn(&bus)
    }
}
