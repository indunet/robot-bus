//! Typed mapper for `sensor_msgs/msg/LaserScan`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn laser_scan_to_bus(
    msg: ros_env::sensor_msgs::msg::LaserScan,
) -> crate::sensor_msgs::msg::v1::LaserScan {
    crate::sensor_msgs::msg::v1::LaserScan {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        angle_min: msg.angle_min,
        angle_max: msg.angle_max,
        angle_increment: msg.angle_increment,
        time_increment: msg.time_increment,
        scan_time: msg.scan_time,
        range_min: msg.range_min,
        range_max: msg.range_max,
        ranges: crate::ros2_bridge::mappers::convert::f32_seq(msg.ranges),
        intensities: crate::ros2_bridge::mappers::convert::f32_seq(msg.intensities),
    }
}

pub(crate) fn laser_scan_to_ros(
    bus: crate::sensor_msgs::msg::v1::LaserScan,
) -> ros_env::sensor_msgs::msg::LaserScan {
    ros_env::sensor_msgs::msg::LaserScan {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        angle_min: bus.angle_min,
        angle_max: bus.angle_max,
        angle_increment: bus.angle_increment,
        time_increment: bus.time_increment,
        scan_time: bus.scan_time,
        range_min: bus.range_min,
        range_max: bus.range_max,
        ranges: bus.ranges,
        intensities: bus.intensities,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsLaserScanMapper;

impl TypedTopicMapper for SensorMsgsLaserScanMapper {
    type Ros = ros_env::sensor_msgs::msg::LaserScan;
    type Bus = crate::sensor_msgs::msg::v1::LaserScan;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(laser_scan_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(laser_scan_to_ros(msg))
    }
}
