//! Typed mapper for `geometry_msgs/msg/PointStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn point_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::PointStamped,
) -> robot_bus::geometry_msgs::msg::v1::PointStamped {
    robot_bus::geometry_msgs::msg::v1::PointStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        point: Some(crate::mappers::geometry_msgs::point::point_to_bus(msg.point)),
    }
}

pub(crate) fn point_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::PointStamped,
) -> ros_env::geometry_msgs::msg::PointStamped {
    ros_env::geometry_msgs::msg::PointStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        point: crate::mappers::geometry_msgs::point::point_to_ros(
            bus.point.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPointStampedMapper;

impl TypedTopicMapper for GeometryMsgsPointStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PointStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::PointStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(point_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(point_stamped_to_ros(msg))
    }
}
