//! Typed mapper for `std_msgs/msg/MultiArrayDimension`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn multi_array_dimension_to_bus(
    msg: ros_env::std_msgs::msg::MultiArrayDimension,
) -> robot_bus::std_msgs::msg::v1::MultiArrayDimension {
    robot_bus::std_msgs::msg::v1::MultiArrayDimension {
        label: crate::mappers::convert::from_ros_string(msg.label),
        size: msg.size.into(),
        stride: msg.stride.into(),
    }
}

pub(crate) fn multi_array_dimension_to_ros(
    bus: robot_bus::std_msgs::msg::v1::MultiArrayDimension,
) -> ros_env::std_msgs::msg::MultiArrayDimension {
    ros_env::std_msgs::msg::MultiArrayDimension {
        label: crate::mappers::convert::to_ros_string(bus.label),
        size: bus.size as _,
        stride: bus.stride as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsMultiArrayDimensionMapper;

impl TypedTopicMapper for StdMsgsMultiArrayDimensionMapper {
    type Ros = ros_env::std_msgs::msg::MultiArrayDimension;
    type Bus = robot_bus::std_msgs::msg::v1::MultiArrayDimension;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(multi_array_dimension_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(multi_array_dimension_to_ros(msg))
    }
}
