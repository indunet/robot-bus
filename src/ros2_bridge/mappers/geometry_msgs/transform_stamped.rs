//! Typed mapper for `geometry_msgs/msg/TransformStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn transform_stamped_to_bus(msg: ros_env::geometry_msgs::msg::TransformStamped) -> crate::geometry_msgs::msg::v1::TransformStamped {
    crate::geometry_msgs::msg::v1::TransformStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        child_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.child_frame_id),
        transform: Some(crate::ros2_bridge::mappers::geometry_msgs::transform::transform_to_bus(msg.transform)),
    }
}

pub(crate) fn transform_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::TransformStamped) -> ros_env::geometry_msgs::msg::TransformStamped {
    ros_env::geometry_msgs::msg::TransformStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        child_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.child_frame_id),
        transform: crate::ros2_bridge::mappers::geometry_msgs::transform::transform_to_ros(bus.transform.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTransformStampedMapper;

impl TypedTopicMapper for GeometryMsgsTransformStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::TransformStamped;
    type Bus = crate::geometry_msgs::msg::v1::TransformStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/TransformStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(transform_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(transform_stamped_to_ros(msg))
    }
}
