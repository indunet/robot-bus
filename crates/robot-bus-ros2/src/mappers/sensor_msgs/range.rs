//! Typed mapper for `sensor_msgs/msg/Range`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn range_to_bus(
    msg: ros_env::sensor_msgs::msg::Range,
) -> robot_bus::sensor_msgs::msg::v1::Range {
    robot_bus::sensor_msgs::msg::v1::Range {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        radiation_type: msg.radiation_type.into(),
        field_of_view: msg.field_of_view,
        min_range: msg.min_range,
        max_range: msg.max_range,
        range: msg.range,
    }
}

// Jazzy adds variance, which has no field in the bus message.
#[allow(clippy::needless_update)]
pub(crate) fn range_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::Range,
) -> ros_env::sensor_msgs::msg::Range {
    ros_env::sensor_msgs::msg::Range {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        radiation_type: bus.radiation_type as _,
        field_of_view: bus.field_of_view,
        min_range: bus.min_range,
        max_range: bus.max_range,
        range: bus.range,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsRangeMapper;

impl TypedTopicMapper for SensorMsgsRangeMapper {
    type Ros = ros_env::sensor_msgs::msg::Range;
    type Bus = robot_bus::sensor_msgs::msg::v1::Range;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(range_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(range_to_ros(msg))
    }
}
