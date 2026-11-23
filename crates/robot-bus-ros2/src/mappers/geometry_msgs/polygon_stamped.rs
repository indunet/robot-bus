//! Typed mapper for `geometry_msgs/msg/PolygonStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn polygon_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::PolygonStamped,
) -> robot_bus::geometry_msgs::msg::v1::PolygonStamped {
    robot_bus::geometry_msgs::msg::v1::PolygonStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        polygon: Some(
            crate::mappers::geometry_msgs::polygon::polygon_to_bus(msg.polygon),
        ),
    }
}

pub(crate) fn polygon_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::PolygonStamped,
) -> ros_env::geometry_msgs::msg::PolygonStamped {
    ros_env::geometry_msgs::msg::PolygonStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        polygon: crate::mappers::geometry_msgs::polygon::polygon_to_ros(
            bus.polygon.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPolygonStampedMapper;

impl TypedTopicMapper for GeometryMsgsPolygonStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PolygonStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::PolygonStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(polygon_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(polygon_stamped_to_ros(msg))
    }
}
