//! Typed mapper for `sensor_msgs/msg/RegionOfInterest`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn region_of_interest_to_bus(
    msg: ros_env::sensor_msgs::msg::RegionOfInterest,
) -> robot_bus::sensor_msgs::msg::v1::RegionOfInterest {
    robot_bus::sensor_msgs::msg::v1::RegionOfInterest {
        x_offset: msg.x_offset.into(),
        y_offset: msg.y_offset.into(),
        height: msg.height.into(),
        width: msg.width.into(),
        do_rectify: msg.do_rectify,
    }
}

pub(crate) fn region_of_interest_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::RegionOfInterest,
) -> ros_env::sensor_msgs::msg::RegionOfInterest {
    ros_env::sensor_msgs::msg::RegionOfInterest {
        x_offset: bus.x_offset as _,
        y_offset: bus.y_offset as _,
        height: bus.height as _,
        width: bus.width as _,
        do_rectify: bus.do_rectify,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsRegionOfInterestMapper;

impl TypedTopicMapper for SensorMsgsRegionOfInterestMapper {
    type Ros = ros_env::sensor_msgs::msg::RegionOfInterest;
    type Bus = robot_bus::sensor_msgs::msg::v1::RegionOfInterest;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(region_of_interest_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(region_of_interest_to_ros(msg))
    }
}
