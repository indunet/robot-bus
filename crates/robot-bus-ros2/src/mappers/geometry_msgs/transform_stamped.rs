//! Typed mapper for `geometry_msgs/msg/TransformStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn transform_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::TransformStamped,
) -> robot_bus::geometry_msgs::msg::v1::TransformStamped {
    robot_bus::geometry_msgs::msg::v1::TransformStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        child_frame_id: crate::mappers::convert::from_ros_string(msg.child_frame_id),
        transform: Some(
            crate::mappers::geometry_msgs::transform::transform_to_bus(msg.transform),
        ),
    }
}

pub(crate) fn transform_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::TransformStamped,
) -> ros_env::geometry_msgs::msg::TransformStamped {
    ros_env::geometry_msgs::msg::TransformStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        child_frame_id: crate::mappers::convert::to_ros_string(bus.child_frame_id),
        transform: crate::mappers::geometry_msgs::transform::transform_to_ros(
            bus.transform.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTransformStampedMapper;

impl TypedTopicMapper for GeometryMsgsTransformStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::TransformStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::TransformStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(transform_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(transform_stamped_to_ros(msg))
    }
}
