//! Typed mapper for `foxglove_msgs/msg/Odometry`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn odometry_to_bus(msg: ros_env::foxglove_msgs::msg::Odometry) -> crate::foxglove_msgs::msg::v1::Odometry {
    crate::foxglove_msgs::msg::v1::Odometry {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        body_frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.body_frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        linear_velocity: msg.linear_velocity.map(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus),
        angular_velocity: msg.angular_velocity.map(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus),
        pose_covariance: crate::ros2_bridge::mappers::convert::f64_seq(msg.pose_covariance),
        velocity_covariance: crate::ros2_bridge::mappers::convert::f64_seq(msg.velocity_covariance),
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn odometry_to_ros(bus: crate::foxglove_msgs::msg::v1::Odometry) -> ros_env::foxglove_msgs::msg::Odometry {
    ros_env::foxglove_msgs::msg::Odometry {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        body_frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.body_frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        linear_velocity: bus.linear_velocity.map(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros),
        angular_velocity: bus.angular_velocity.map(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros),
        pose_covariance: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.pose_covariance),
        velocity_covariance: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.velocity_covariance),
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsOdometryMapper;

impl TypedTopicMapper for FoxgloveMsgsOdometryMapper {
    type Ros = ros_env::foxglove_msgs::msg::Odometry;
    type Bus = crate::foxglove_msgs::msg::v1::Odometry;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Odometry"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(odometry_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(odometry_to_ros(msg))
    }
}
