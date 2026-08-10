//! Mapper for `visualization_msgs/msg/InteractiveMarkerUpdate`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn interactive_marker_update_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerUpdate> {
    Ok(
        crate::visualization_msgs::msg::v1::InteractiveMarkerUpdate {
            server_id: read_string(view, "server_id")?,
            seq_num: read_u64(view, "seq_num")?,
            r#type: read_u32(view, "type")?,
            markers: read_message_seq(view, "markers", super::interactive_marker::interactive_marker_from_view)?,
            poses: read_message_seq(view, "poses", super::interactive_marker_pose::interactive_marker_pose_from_view)?,
            erases: read_string_seq(view, "erases")?,
        },
    )
}

pub(crate) fn interactive_marker_update_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerUpdate,
) -> Result<()> {
    write_string(view, "server_id", &bus.server_id)?;
    write_u64(view, "seq_num", bus.seq_num)?;
    write_u32(view, "type", bus.r#type)?;
    write_message_seq(view, "markers", &bus.markers, super::interactive_marker::interactive_marker_write)?;
    write_message_seq(view, "poses", &bus.poses, super::interactive_marker_pose::interactive_marker_pose_write)?;
    write_string_seq(view, "erases", &bus.erases)?;
    Ok(())
}

pub(crate) fn interactive_marker_update_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerUpdate> {
    interactive_marker_update_from_view(&msg.view())
}

pub(crate) fn interactive_marker_update_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerUpdate,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/InteractiveMarkerUpdate")?;
    interactive_marker_update_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsInteractiveMarkerUpdateMapper;
impl TopicMapper for VisualizationMsgsInteractiveMarkerUpdateMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/InteractiveMarkerUpdate"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interactive_marker_update_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::InteractiveMarkerUpdate as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode visualization_msgs/msg/InteractiveMarkerUpdate: {e}"
                ))
            })?;
        interactive_marker_update_bus_to_dyn(&bus)
    }
}
