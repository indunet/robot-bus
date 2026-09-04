//! Typed mapper for `nav2_msgs/msg/RouteEdge`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn route_edge_to_bus(msg: ros_env::nav2_msgs::msg::RouteEdge) -> crate::nav2_msgs::msg::v1::RouteEdge {
    crate::nav2_msgs::msg::v1::RouteEdge {
        edgeid: msg.edgeid.into(),
        start: msg.start.into(),
        end: msg.end.into(),
    }
}

pub(crate) fn route_edge_to_ros(bus: crate::nav2_msgs::msg::v1::RouteEdge) -> ros_env::nav2_msgs::msg::RouteEdge {
    ros_env::nav2_msgs::msg::RouteEdge {
        edgeid: bus.edgeid as _,
        start: bus.start as _,
        end: bus.end as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsRouteEdgeMapper;

impl TypedTopicMapper for Nav2MsgsRouteEdgeMapper {
    type Ros = ros_env::nav2_msgs::msg::RouteEdge;
    type Bus = crate::nav2_msgs::msg::v1::RouteEdge;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(route_edge_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(route_edge_to_ros(msg))
    }
}
