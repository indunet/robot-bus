//! Mapper for `geometry_msgs/msg/Pose`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn pose_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Pose> {
    Ok(crate::geometry_msgs::msg::v1::Pose {
        position: nested_view(view, "position")?
            .as_ref()
            .map(super::point::point_from_view)
            .transpose()?,
        orientation: nested_view(view, "orientation")?
            .as_ref()
            .map(super::quaternion::quaternion_from_view)
            .transpose()?,
    })
}

pub(crate) fn pose_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Pose,
) -> Result<()> {
    if let Some(v) = &bus.position {
        with_nested_mut(view, "position", |nested| {
            super::point::point_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.orientation {
        with_nested_mut(view, "orientation", |nested| {
            super::quaternion::quaternion_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn pose_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Pose> {
    pose_from_view(&msg.view())
}

pub(crate) fn pose_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Pose,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Pose")?;
    pose_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPoseMapper;
impl TopicMapper for GeometryMsgsPoseMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Pose"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(pose_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Pose as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Pose: {e}")))?;
        pose_bus_to_dyn(&bus)
    }
}
