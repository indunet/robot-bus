//! Typed mapper for `std_msgs/msg/MultiArrayDimension`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn multi_array_dimension_to_bus(msg: ros_env::std_msgs::msg::MultiArrayDimension) -> crate::std_msgs::msg::v1::MultiArrayDimension {
    crate::std_msgs::msg::v1::MultiArrayDimension {
        label: crate::ros2_bridge::mappers::convert::from_ros_string(msg.label),
        size: msg.size,
        stride: msg.stride,
    }
}

pub(crate) fn multi_array_dimension_to_ros(bus: crate::std_msgs::msg::v1::MultiArrayDimension) -> ros_env::std_msgs::msg::MultiArrayDimension {
    ros_env::std_msgs::msg::MultiArrayDimension {
        label: crate::ros2_bridge::mappers::convert::to_ros_string(bus.label),
        size: bus.size,
        stride: bus.stride,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsMultiArrayDimensionMapper;

impl TypedTopicMapper for StdMsgsMultiArrayDimensionMapper {
    type Ros = ros_env::std_msgs::msg::MultiArrayDimension;
    type Bus = crate::std_msgs::msg::v1::MultiArrayDimension;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(multi_array_dimension_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(multi_array_dimension_to_ros(msg))
    }
}
