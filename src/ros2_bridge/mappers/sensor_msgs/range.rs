//! Typed mapper for `sensor_msgs/msg/Range`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn range_to_bus(
    msg: ros_env::sensor_msgs::msg::Range,
) -> crate::sensor_msgs::msg::v1::Range {
    crate::sensor_msgs::msg::v1::Range {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        radiation_type: msg.radiation_type.into(),
        field_of_view: msg.field_of_view,
        min_range: msg.min_range,
        max_range: msg.max_range,
        range: msg.range,
    }
}

pub(crate) fn range_to_ros(
    bus: crate::sensor_msgs::msg::v1::Range,
) -> ros_env::sensor_msgs::msg::Range {
    ros_env::sensor_msgs::msg::Range {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        radiation_type: bus.radiation_type as _,
        field_of_view: bus.field_of_view,
        min_range: bus.min_range,
        max_range: bus.max_range,
        range: bus.range,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsRangeMapper;

impl TypedTopicMapper for SensorMsgsRangeMapper {
    type Ros = ros_env::sensor_msgs::msg::Range;
    type Bus = crate::sensor_msgs::msg::v1::Range;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(range_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(range_to_ros(msg))
    }
}
