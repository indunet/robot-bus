//! Typed mapper for `foxglove_msgs/msg/LaserScan`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn laser_scan_to_bus(msg: ros_env::foxglove_msgs::msg::LaserScan) -> crate::foxglove_msgs::msg::v1::LaserScan {
    crate::foxglove_msgs::msg::v1::LaserScan {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        start_angle: msg.start_angle,
        end_angle: msg.end_angle,
        ranges: crate::ros2_bridge::mappers::convert::f64_seq(msg.ranges),
        intensities: crate::ros2_bridge::mappers::convert::f64_seq(msg.intensities),
    }
}

pub(crate) fn laser_scan_to_ros(bus: crate::foxglove_msgs::msg::v1::LaserScan) -> ros_env::foxglove_msgs::msg::LaserScan {
    ros_env::foxglove_msgs::msg::LaserScan {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        start_angle: bus.start_angle,
        end_angle: bus.end_angle,
        ranges: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.ranges),
        intensities: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.intensities),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsLaserScanMapper;

impl TypedTopicMapper for FoxgloveMsgsLaserScanMapper {
    type Ros = ros_env::foxglove_msgs::msg::LaserScan;
    type Bus = crate::foxglove_msgs::msg::v1::LaserScan;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LaserScan"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(laser_scan_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(laser_scan_to_ros(msg))
    }
}
