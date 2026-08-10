//! Mapper for `visualization_msgs/msg/MarkerArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn marker_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::MarkerArray> {
    Ok(crate::visualization_msgs::msg::v1::MarkerArray {
        markers: read_message_seq(view, "markers", super::marker::marker_from_view)?,
    })
}

pub(crate) fn marker_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::MarkerArray,
) -> Result<()> {
    write_message_seq(view, "markers", &bus.markers, super::marker::marker_write)?;
    Ok(())
}

pub(crate) fn marker_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::MarkerArray> {
    marker_array_from_view(&msg.view())
}

pub(crate) fn marker_array_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::MarkerArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/MarkerArray")?;
    marker_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsMarkerArrayMapper;
impl TopicMapper for VisualizationMsgsMarkerArrayMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/MarkerArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(marker_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::MarkerArray as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode visualization_msgs/msg/MarkerArray: {e}"))
                })?;
        marker_array_bus_to_dyn(&bus)
    }
}
