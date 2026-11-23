//! Typed mapper for `sensor_msgs/msg/NavSatFix`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn nav_sat_fix_to_bus(
    msg: ros_env::sensor_msgs::msg::NavSatFix,
) -> robot_bus::sensor_msgs::msg::v1::NavSatFix {
    robot_bus::sensor_msgs::msg::v1::NavSatFix {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        status: Some(
            crate::mappers::sensor_msgs::nav_sat_status::nav_sat_status_to_bus(
                msg.status,
            ),
        ),
        latitude: msg.latitude,
        longitude: msg.longitude,
        altitude: msg.altitude,
        position_covariance: crate::mappers::convert::f64_seq(msg.position_covariance),
        position_covariance_type: msg.position_covariance_type.into(),
    }
}

pub(crate) fn nav_sat_fix_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::NavSatFix,
) -> ros_env::sensor_msgs::msg::NavSatFix {
    ros_env::sensor_msgs::msg::NavSatFix {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        status: crate::mappers::sensor_msgs::nav_sat_status::nav_sat_status_to_ros(
            bus.status.unwrap_or_default(),
        ),
        latitude: bus.latitude,
        longitude: bus.longitude,
        altitude: bus.altitude,
        position_covariance: crate::mappers::convert::FromF64Seq::from_f64_seq(
            bus.position_covariance,
        ),
        position_covariance_type: bus.position_covariance_type as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsNavSatFixMapper;

impl TypedTopicMapper for SensorMsgsNavSatFixMapper {
    type Ros = ros_env::sensor_msgs::msg::NavSatFix;
    type Bus = robot_bus::sensor_msgs::msg::v1::NavSatFix;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(nav_sat_fix_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(nav_sat_fix_to_ros(msg))
    }
}
