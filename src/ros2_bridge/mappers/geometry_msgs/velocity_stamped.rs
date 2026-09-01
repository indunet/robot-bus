//! Typed mapper for `geometry_msgs/msg/VelocityStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn velocity_stamped_to_bus(msg: ros_env::geometry_msgs::msg::VelocityStamped) -> crate::geometry_msgs::msg::v1::VelocityStamped {
    crate::geometry_msgs::msg::v1::VelocityStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        body_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.body_frame_id),
        reference_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.reference_frame_id),
        velocity: Some(crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_bus(msg.velocity)),
    }
}

pub(crate) fn velocity_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::VelocityStamped) -> ros_env::geometry_msgs::msg::VelocityStamped {
    ros_env::geometry_msgs::msg::VelocityStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        body_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.body_frame_id),
        reference_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.reference_frame_id),
        velocity: crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_ros(bus.velocity.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsVelocityStampedMapper;

impl TypedTopicMapper for GeometryMsgsVelocityStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::VelocityStamped;
    type Bus = crate::geometry_msgs::msg::v1::VelocityStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(velocity_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(velocity_stamped_to_ros(msg))
    }
}
