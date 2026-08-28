//! Typed mapper for `visualization_msgs/msg/UVCoordinate`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uv_coordinate_to_bus(msg: ros_env::visualization_msgs::msg::UVCoordinate) -> crate::visualization_msgs::msg::v1::UvCoordinate {
    crate::visualization_msgs::msg::v1::UvCoordinate {
        u: msg.u,
        v: msg.v,
    }
}

pub(crate) fn uv_coordinate_to_ros(bus: crate::visualization_msgs::msg::v1::UvCoordinate) -> ros_env::visualization_msgs::msg::UVCoordinate {
    ros_env::visualization_msgs::msg::UVCoordinate {
        u: bus.u,
        v: bus.v,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsUvCoordinateMapper;

impl TypedTopicMapper for VisualizationMsgsUvCoordinateMapper {
    type Ros = ros_env::visualization_msgs::msg::UVCoordinate;
    type Bus = crate::visualization_msgs::msg::v1::UvCoordinate;

    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/UVCoordinate"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uv_coordinate_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uv_coordinate_to_ros(msg))
    }
}
