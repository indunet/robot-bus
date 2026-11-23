//! Typed mapper for `sensor_msgs/msg/LaserScan`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn laser_scan_to_bus(
    msg: ros_env::sensor_msgs::msg::LaserScan,
) -> robot_bus::sensor_msgs::msg::v1::LaserScan {
    robot_bus::sensor_msgs::msg::v1::LaserScan {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        angle_min: msg.angle_min,
        angle_max: msg.angle_max,
        angle_increment: msg.angle_increment,
        time_increment: msg.time_increment,
        scan_time: msg.scan_time,
        range_min: msg.range_min,
        range_max: msg.range_max,
        ranges: crate::mappers::convert::f32_seq(msg.ranges),
        intensities: crate::mappers::convert::f32_seq(msg.intensities),
    }
}

pub(crate) fn laser_scan_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::LaserScan,
) -> ros_env::sensor_msgs::msg::LaserScan {
    ros_env::sensor_msgs::msg::LaserScan {
        header: crate::mappers::std_msgs::header::header_to_ros(
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
    type Bus = robot_bus::sensor_msgs::msg::v1::LaserScan;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(laser_scan_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(laser_scan_to_ros(msg))
    }
}
