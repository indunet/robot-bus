//! Typed mapper for `nav2_msgs/msg/EdgeCost`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn edge_cost_to_bus(msg: ros_env::nav2_msgs::msg::EdgeCost) -> crate::nav2_msgs::msg::v1::EdgeCost {
    crate::nav2_msgs::msg::v1::EdgeCost {
        edgeid: msg.edgeid,
        cost: msg.cost,
    }
}

pub(crate) fn edge_cost_to_ros(bus: crate::nav2_msgs::msg::v1::EdgeCost) -> ros_env::nav2_msgs::msg::EdgeCost {
    ros_env::nav2_msgs::msg::EdgeCost {
        edgeid: bus.edgeid,
        cost: bus.cost,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsEdgeCostMapper;

impl TypedTopicMapper for Nav2MsgsEdgeCostMapper {
    type Ros = ros_env::nav2_msgs::msg::EdgeCost;
    type Bus = crate::nav2_msgs::msg::v1::EdgeCost;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/EdgeCost"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(edge_cost_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(edge_cost_to_ros(msg))
    }
}
