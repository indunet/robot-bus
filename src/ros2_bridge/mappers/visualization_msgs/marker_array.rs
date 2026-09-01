//! Typed mapper for `visualization_msgs/msg/MarkerArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn marker_array_to_bus(msg: ros_env::visualization_msgs::msg::MarkerArray) -> crate::visualization_msgs::msg::v1::MarkerArray {
    crate::visualization_msgs::msg::v1::MarkerArray {
        markers: msg.markers.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::marker::marker_to_bus).collect(),
    }
}

pub(crate) fn marker_array_to_ros(bus: crate::visualization_msgs::msg::v1::MarkerArray) -> ros_env::visualization_msgs::msg::MarkerArray {
    ros_env::visualization_msgs::msg::MarkerArray {
        markers: bus.markers.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::marker::marker_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsMarkerArrayMapper;

impl TypedTopicMapper for VisualizationMsgsMarkerArrayMapper {
    type Ros = ros_env::visualization_msgs::msg::MarkerArray;
    type Bus = crate::visualization_msgs::msg::v1::MarkerArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(marker_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(marker_array_to_ros(msg))
    }
}
