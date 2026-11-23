//! Typed mapper for `geometry_msgs/msg/PolygonInstanceStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn polygon_instance_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::PolygonInstanceStamped,
) -> robot_bus::geometry_msgs::msg::v1::PolygonInstanceStamped {
    robot_bus::geometry_msgs::msg::v1::PolygonInstanceStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        polygon: Some(
            crate::mappers::geometry_msgs::polygon_instance::polygon_instance_to_bus(
                msg.polygon,
            ),
        ),
    }
}

pub(crate) fn polygon_instance_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::PolygonInstanceStamped,
) -> ros_env::geometry_msgs::msg::PolygonInstanceStamped {
    ros_env::geometry_msgs::msg::PolygonInstanceStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        polygon:
            crate::mappers::geometry_msgs::polygon_instance::polygon_instance_to_ros(
                bus.polygon.unwrap_or_default(),
            ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPolygonInstanceStampedMapper;

impl TypedTopicMapper for GeometryMsgsPolygonInstanceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PolygonInstanceStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::PolygonInstanceStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(polygon_instance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(polygon_instance_stamped_to_ros(msg))
    }
}
