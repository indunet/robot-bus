//! Mapper for `nav2_msgs/msg/RouteNode`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn route_node_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::RouteNode> {
    Ok(crate::nav2_msgs::msg::v1::RouteNode {
        nodeid: read_u32(view, "nodeid")?,
        position: nested_view(view, "position")?
            .as_ref()
            .map(super::super::geometry_msgs::point::point_from_view)
            .transpose()?,
    })
}

pub(crate) fn route_node_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::RouteNode,
) -> Result<()> {
    write_u32(view, "nodeid", bus.nodeid)?;
    if let Some(v) = &bus.position {
        with_nested_mut(view, "position", |nested| {
            super::super::geometry_msgs::point::point_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn route_node_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::RouteNode> {
    route_node_from_view(&msg.view())
}

pub(crate) fn route_node_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::RouteNode,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/RouteNode")?;
    route_node_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsRouteNodeMapper;
impl TopicMapper for Nav2MsgsRouteNodeMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/RouteNode"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(route_node_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::RouteNode as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/RouteNode: {e}")))?;
        route_node_bus_to_dyn(&bus)
    }
}
