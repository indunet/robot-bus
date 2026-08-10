//! Mapper for `visualization_msgs/msg/InteractiveMarkerInit`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn interactive_marker_init_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerInit> {
    Ok(crate::visualization_msgs::msg::v1::InteractiveMarkerInit {
        server_id: read_string(view, "server_id")?,
        seq_num: read_u64(view, "seq_num")?,
        markers: read_message_seq(view, "markers", super::interactive_marker::interactive_marker_from_view)?,
    })
}

pub(crate) fn interactive_marker_init_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerInit,
) -> Result<()> {
    write_string(view, "server_id", &bus.server_id)?;
    write_u64(view, "seq_num", bus.seq_num)?;
    write_message_seq(view, "markers", &bus.markers, super::interactive_marker::interactive_marker_write)?;
    Ok(())
}

pub(crate) fn interactive_marker_init_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::InteractiveMarkerInit> {
    interactive_marker_init_from_view(&msg.view())
}

pub(crate) fn interactive_marker_init_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::InteractiveMarkerInit,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/InteractiveMarkerInit")?;
    interactive_marker_init_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsInteractiveMarkerInitMapper;
impl TopicMapper for VisualizationMsgsInteractiveMarkerInitMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/InteractiveMarkerInit"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interactive_marker_init_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::InteractiveMarkerInit as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode visualization_msgs/msg/InteractiveMarkerInit: {e}"
                ))
            })?;
        interactive_marker_init_bus_to_dyn(&bus)
    }
}
