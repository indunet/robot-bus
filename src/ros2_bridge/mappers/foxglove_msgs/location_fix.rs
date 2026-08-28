//! Typed mapper for `foxglove_msgs/msg/LocationFix`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn location_fix_to_bus(msg: ros_env::foxglove_msgs::msg::LocationFix) -> crate::foxglove_msgs::msg::v1::LocationFix {
    crate::foxglove_msgs::msg::v1::LocationFix {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        latitude: msg.latitude,
        longitude: msg.longitude,
        altitude: msg.altitude,
        position_covariance: crate::ros2_bridge::mappers::convert::f64_seq(msg.position_covariance),
        position_covariance_type: msg.position_covariance_type as i32,
        heading: msg.heading,
        velocity: msg.velocity.map(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus),
        color: msg.color.map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus),
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
    }
}

pub(crate) fn location_fix_to_ros(bus: crate::foxglove_msgs::msg::v1::LocationFix) -> ros_env::foxglove_msgs::msg::LocationFix {
    ros_env::foxglove_msgs::msg::LocationFix {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        latitude: bus.latitude,
        longitude: bus.longitude,
        altitude: bus.altitude,
        position_covariance: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.position_covariance),
        position_covariance_type: bus.position_covariance_type as i32,
        heading: bus.heading,
        velocity: bus.velocity.map(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros),
        color: bus.color.map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros),
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsLocationFixMapper;

impl TypedTopicMapper for FoxgloveMsgsLocationFixMapper {
    type Ros = ros_env::foxglove_msgs::msg::LocationFix;
    type Bus = crate::foxglove_msgs::msg::v1::LocationFix;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LocationFix"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(location_fix_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(location_fix_to_ros(msg))
    }
}
