//! Typed mapper for `visualization_msgs/msg/InteractiveMarkerInit`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn interactive_marker_init_to_bus(msg: ros_env::visualization_msgs::msg::InteractiveMarkerInit) -> crate::visualization_msgs::msg::v1::InteractiveMarkerInit {
    crate::visualization_msgs::msg::v1::InteractiveMarkerInit {
        server_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.server_id),
        seq_num: msg.seq_num,
        markers: msg.markers.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::interactive_marker::interactive_marker_to_bus).collect(),
    }
}

pub(crate) fn interactive_marker_init_to_ros(bus: crate::visualization_msgs::msg::v1::InteractiveMarkerInit) -> ros_env::visualization_msgs::msg::InteractiveMarkerInit {
    ros_env::visualization_msgs::msg::InteractiveMarkerInit {
        server_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.server_id),
        seq_num: bus.seq_num,
        markers: bus.markers.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::interactive_marker::interactive_marker_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsInteractiveMarkerInitMapper;

impl TypedTopicMapper for VisualizationMsgsInteractiveMarkerInitMapper {
    type Ros = ros_env::visualization_msgs::msg::InteractiveMarkerInit;
    type Bus = crate::visualization_msgs::msg::v1::InteractiveMarkerInit;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(interactive_marker_init_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(interactive_marker_init_to_ros(msg))
    }
}
