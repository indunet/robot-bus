//! Typed mapper for `geometry_msgs/msg/VelocityWithCovarianceStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn velocity_with_covariance_stamped_to_bus(msg: ros_env::geometry_msgs::msg::VelocityWithCovarianceStamped) -> crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped {
    crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        body_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.body_frame_id),
        reference_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.reference_frame_id),
        velocity: Some(crate::ros2_bridge::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_bus(msg.velocity)),
    }
}

pub(crate) fn velocity_with_covariance_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped) -> ros_env::geometry_msgs::msg::VelocityWithCovarianceStamped {
    ros_env::geometry_msgs::msg::VelocityWithCovarianceStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        body_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.body_frame_id),
        reference_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.reference_frame_id),
        velocity: crate::ros2_bridge::mappers::geometry_msgs::twist_with_covariance::twist_with_covariance_to_ros(bus.velocity.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsVelocityWithCovarianceStampedMapper;

impl TypedTopicMapper for GeometryMsgsVelocityWithCovarianceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::VelocityWithCovarianceStamped;
    type Bus = crate::geometry_msgs::msg::v1::VelocityWithCovarianceStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/VelocityWithCovarianceStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(velocity_with_covariance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(velocity_with_covariance_stamped_to_ros(msg))
    }
}
