//! Typed mapper for `sensor_msgs/msg/PointField`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_field_to_bus(
    msg: ros_env::sensor_msgs::msg::PointField,
) -> crate::sensor_msgs::msg::v1::PointField {
    crate::sensor_msgs::msg::v1::PointField {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        offset: msg.offset.into(),
        datatype: u32::from(msg.datatype),
        count: msg.count.into(),
    }
}

pub(crate) fn point_field_to_ros(
    bus: crate::sensor_msgs::msg::v1::PointField,
) -> ros_env::sensor_msgs::msg::PointField {
    ros_env::sensor_msgs::msg::PointField {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        offset: bus.offset as _,
        datatype: bus.datatype as u8,
        count: bus.count as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsPointFieldMapper;

impl TypedTopicMapper for SensorMsgsPointFieldMapper {
    type Ros = ros_env::sensor_msgs::msg::PointField;
    type Bus = crate::sensor_msgs::msg::v1::PointField;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_field_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_field_to_ros(msg))
    }
}
