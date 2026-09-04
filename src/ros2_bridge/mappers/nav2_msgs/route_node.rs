//! Typed mapper for `nav2_msgs/msg/RouteNode`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn route_node_to_bus(msg: ros_env::nav2_msgs::msg::RouteNode) -> crate::nav2_msgs::msg::v1::RouteNode {
    crate::nav2_msgs::msg::v1::RouteNode {
        nodeid: msg.nodeid.into(),
        position: Some(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus(msg.position)),
    }
}

pub(crate) fn route_node_to_ros(bus: crate::nav2_msgs::msg::v1::RouteNode) -> ros_env::nav2_msgs::msg::RouteNode {
    ros_env::nav2_msgs::msg::RouteNode {
        nodeid: bus.nodeid as _,
        position: crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros(bus.position.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsRouteNodeMapper;

impl TypedTopicMapper for Nav2MsgsRouteNodeMapper {
    type Ros = ros_env::nav2_msgs::msg::RouteNode;
    type Bus = crate::nav2_msgs::msg::v1::RouteNode;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(route_node_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(route_node_to_ros(msg))
    }
}
