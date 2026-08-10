//! Mapper for `nav2_msgs/msg/Route`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn route_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::Route> {
    Ok(crate::nav2_msgs::msg::v1::Route {
        nodes: read_message_seq(view, "nodes", super::route_node::route_node_from_view)?,
        edges: read_message_seq(view, "edges", super::route_edge::route_edge_from_view)?,
    })
}

pub(crate) fn route_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::Route,
) -> Result<()> {
    write_message_seq(view, "nodes", &bus.nodes, super::route_node::route_node_write)?;
    write_message_seq(view, "edges", &bus.edges, super::route_edge::route_edge_write)?;
    Ok(())
}

pub(crate) fn route_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::Route> {
    route_from_view(&msg.view())
}

pub(crate) fn route_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::Route,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/Route")?;
    route_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsRouteMapper;
impl TopicMapper for Nav2MsgsRouteMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/Route"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(route_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::Route as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/Route: {e}")))?;
        route_bus_to_dyn(&bus)
    }
}
