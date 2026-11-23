//! Typed mapper for `sensor_msgs/msg/MultiEchoLaserScan`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn multi_echo_laser_scan_to_bus(
    msg: ros_env::sensor_msgs::msg::MultiEchoLaserScan,
) -> robot_bus::sensor_msgs::msg::v1::MultiEchoLaserScan {
    robot_bus::sensor_msgs::msg::v1::MultiEchoLaserScan {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        angle_min: msg.angle_min,
        angle_max: msg.angle_max,
        angle_increment: msg.angle_increment,
        time_increment: msg.time_increment,
        scan_time: msg.scan_time,
        range_min: msg.range_min,
        range_max: msg.range_max,
        ranges: msg
            .ranges
            .into_iter()
            .map(crate::mappers::sensor_msgs::laser_echo::laser_echo_to_bus)
            .collect(),
        intensities: msg
            .intensities
            .into_iter()
            .map(crate::mappers::sensor_msgs::laser_echo::laser_echo_to_bus)
            .collect(),
    }
}

pub(crate) fn multi_echo_laser_scan_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::MultiEchoLaserScan,
) -> ros_env::sensor_msgs::msg::MultiEchoLaserScan {
    ros_env::sensor_msgs::msg::MultiEchoLaserScan {
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
        ranges: bus
            .ranges
            .into_iter()
            .map(crate::mappers::sensor_msgs::laser_echo::laser_echo_to_ros)
            .collect(),
        intensities: bus
            .intensities
            .into_iter()
            .map(crate::mappers::sensor_msgs::laser_echo::laser_echo_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsMultiEchoLaserScanMapper;

impl TypedTopicMapper for SensorMsgsMultiEchoLaserScanMapper {
    type Ros = ros_env::sensor_msgs::msg::MultiEchoLaserScan;
    type Bus = robot_bus::sensor_msgs::msg::v1::MultiEchoLaserScan;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(multi_echo_laser_scan_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(multi_echo_laser_scan_to_ros(msg))
    }
}
