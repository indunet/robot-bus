//! Typed mapper for `sensor_msgs/msg/RegionOfInterest`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn region_of_interest_to_bus(msg: ros_env::sensor_msgs::msg::RegionOfInterest) -> crate::sensor_msgs::msg::v1::RegionOfInterest {
    crate::sensor_msgs::msg::v1::RegionOfInterest {
        x_offset: msg.x_offset,
        y_offset: msg.y_offset,
        height: msg.height,
        width: msg.width,
        do_rectify: msg.do_rectify,
    }
}

pub(crate) fn region_of_interest_to_ros(bus: crate::sensor_msgs::msg::v1::RegionOfInterest) -> ros_env::sensor_msgs::msg::RegionOfInterest {
    ros_env::sensor_msgs::msg::RegionOfInterest {
        x_offset: bus.x_offset,
        y_offset: bus.y_offset,
        height: bus.height,
        width: bus.width,
        do_rectify: bus.do_rectify,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsRegionOfInterestMapper;

impl TypedTopicMapper for SensorMsgsRegionOfInterestMapper {
    type Ros = ros_env::sensor_msgs::msg::RegionOfInterest;
    type Bus = crate::sensor_msgs::msg::v1::RegionOfInterest;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(region_of_interest_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(region_of_interest_to_ros(msg))
    }
}
