//! Mapper for `nav2_msgs/msg/RouteEdge`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn route_edge_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::RouteEdge> {
    Ok(crate::nav2_msgs::msg::v1::RouteEdge {
        edgeid: read_u32(view, "edgeid")?,
        start: read_u32(view, "start")?,
        end: read_u32(view, "end")?,
    })
}

pub(crate) fn route_edge_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::RouteEdge,
) -> Result<()> {
    write_u32(view, "edgeid", bus.edgeid)?;
    write_u32(view, "start", bus.start)?;
    write_u32(view, "end", bus.end)?;
    Ok(())
}

pub(crate) fn route_edge_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::RouteEdge> {
    route_edge_from_view(&msg.view())
}

pub(crate) fn route_edge_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::RouteEdge,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/RouteEdge")?;
    route_edge_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsRouteEdgeMapper;
impl TopicMapper for Nav2MsgsRouteEdgeMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/RouteEdge"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(route_edge_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::RouteEdge as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/RouteEdge: {e}")))?;
        route_edge_bus_to_dyn(&bus)
    }
}
