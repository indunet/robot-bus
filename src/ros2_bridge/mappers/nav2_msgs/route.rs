//! Typed mapper for `nav2_msgs/msg/Route`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn route_to_bus(msg: ros_env::nav2_msgs::msg::Route) -> crate::nav2_msgs::msg::v1::Route {
    crate::nav2_msgs::msg::v1::Route {
        nodes: msg.nodes.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::route_node::route_node_to_bus).collect(),
        edges: msg.edges.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::route_edge::route_edge_to_bus).collect(),
    }
}

pub(crate) fn route_to_ros(bus: crate::nav2_msgs::msg::v1::Route) -> ros_env::nav2_msgs::msg::Route {
    ros_env::nav2_msgs::msg::Route {
        nodes: bus.nodes.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::route_node::route_node_to_ros).collect(),
        edges: bus.edges.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::route_edge::route_edge_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsRouteMapper;

impl TypedTopicMapper for Nav2MsgsRouteMapper {
    type Ros = ros_env::nav2_msgs::msg::Route;
    type Bus = crate::nav2_msgs::msg::v1::Route;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/Route"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(route_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(route_to_ros(msg))
    }
}
