//! Typed mapper for `visualization_msgs/msg/UVCoordinate`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn uv_coordinate_to_bus(
    msg: ros_env::visualization_msgs::msg::UVCoordinate,
) -> robot_bus::visualization_msgs::msg::v1::UvCoordinate {
    robot_bus::visualization_msgs::msg::v1::UvCoordinate { u: msg.u, v: msg.v }
}

pub(crate) fn uv_coordinate_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::UvCoordinate,
) -> ros_env::visualization_msgs::msg::UVCoordinate {
    ros_env::visualization_msgs::msg::UVCoordinate { u: bus.u, v: bus.v }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsUvCoordinateMapper;

impl TypedTopicMapper for VisualizationMsgsUvCoordinateMapper {
    type Ros = ros_env::visualization_msgs::msg::UVCoordinate;
    type Bus = robot_bus::visualization_msgs::msg::v1::UvCoordinate;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(uv_coordinate_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(uv_coordinate_to_ros(msg))
    }
}
