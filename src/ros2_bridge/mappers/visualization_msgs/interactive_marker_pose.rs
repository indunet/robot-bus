//! Mapper for `visualization_msgs/msg/InteractiveMarkerPose`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn interactive_marker_pose_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerPose> {
    Ok(crate::visualization_msgs::msg::v1::InteractiveMarkerPose {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
        name: read_string(view, "name")?,
    })
}

pub(crate) fn interactive_marker_pose_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerPose,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    write_string(view, "name", &bus.name)?;
    Ok(())
}

pub(crate) fn interactive_marker_pose_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerPose> {
    interactive_marker_pose_from_view(&msg.view())
}

pub(crate) fn interactive_marker_pose_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerPose,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/InteractiveMarkerPose")?;
    interactive_marker_pose_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsInteractiveMarkerPoseMapper;
impl TopicMapper for VisualizationMsgsInteractiveMarkerPoseMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/InteractiveMarkerPose"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interactive_marker_pose_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::InteractiveMarkerPose as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode visualization_msgs/msg/InteractiveMarkerPose: {e}"
                ))
            })?;
        interactive_marker_pose_bus_to_dyn(&bus)
    }
}
