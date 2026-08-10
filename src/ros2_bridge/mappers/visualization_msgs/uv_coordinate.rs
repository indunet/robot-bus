//! Mapper for `visualization_msgs/msg/UVCoordinate`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn uv_coordinate_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::visualization_msgs::msg::v1::UvCoordinate> {
    Ok(crate::visualization_msgs::msg::v1::UvCoordinate {
        u: read_f32(view, "u")?,
        v: read_f32(view, "v")?,
    })
}

pub(crate) fn uv_coordinate_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::visualization_msgs::msg::v1::UvCoordinate,
) -> Result<()> {
    write_f32(view, "u", bus.u)?;
    write_f32(view, "v", bus.v)?;
    Ok(())
}

pub(crate) fn uv_coordinate_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::visualization_msgs::msg::v1::UvCoordinate> {
    uv_coordinate_from_view(&msg.view())
}

pub(crate) fn uv_coordinate_bus_to_dyn(
    bus: &crate::visualization_msgs::msg::v1::UvCoordinate,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("visualization_msgs/msg/UVCoordinate")?;
    uv_coordinate_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct VisualizationMsgsUvCoordinateMapper;
impl TopicMapper for VisualizationMsgsUvCoordinateMapper {
    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/UVCoordinate"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(uv_coordinate_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::visualization_msgs::msg::v1::UvCoordinate as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode visualization_msgs/msg/UVCoordinate: {e}"))
                })?;
        uv_coordinate_bus_to_dyn(&bus)
    }
}
