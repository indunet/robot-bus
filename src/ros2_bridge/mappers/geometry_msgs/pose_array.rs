//! Mapper for `geometry_msgs/msg/PoseArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn pose_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::PoseArray> {
    Ok(crate::geometry_msgs::msg::v1::PoseArray {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        poses: read_message_seq(view, "poses", super::pose::pose_from_view)?,
    })
}

pub(crate) fn pose_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::PoseArray,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(view, "poses", &bus.poses, super::pose::pose_write)?;
    Ok(())
}

pub(crate) fn pose_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::PoseArray> {
    pose_array_from_view(&msg.view())
}

pub(crate) fn pose_array_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::PoseArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/PoseArray")?;
    pose_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPoseArrayMapper;
impl TopicMapper for GeometryMsgsPoseArrayMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PoseArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(pose_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::PoseArray as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/PoseArray: {e}")))?;
        pose_array_bus_to_dyn(&bus)
    }
}
