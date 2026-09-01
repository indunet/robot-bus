//! Typed mapper for `sensor_msgs/msg/NavSatStatus`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn nav_sat_status_to_bus(msg: ros_env::sensor_msgs::msg::NavSatStatus) -> crate::sensor_msgs::msg::v1::NavSatStatus {
    crate::sensor_msgs::msg::v1::NavSatStatus {
        status: i32::from(msg.status),
        service: u32::from(msg.service),
    }
}

pub(crate) fn nav_sat_status_to_ros(bus: crate::sensor_msgs::msg::v1::NavSatStatus) -> ros_env::sensor_msgs::msg::NavSatStatus {
    ros_env::sensor_msgs::msg::NavSatStatus {
        status: bus.status as i8,
        service: bus.service as u16,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsNavSatStatusMapper;

impl TypedTopicMapper for SensorMsgsNavSatStatusMapper {
    type Ros = ros_env::sensor_msgs::msg::NavSatStatus;
    type Bus = crate::sensor_msgs::msg::v1::NavSatStatus;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(nav_sat_status_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(nav_sat_status_to_ros(msg))
    }
}
