//! Typed mapper for `std_msgs/msg/MultiArrayLayout`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn multi_array_layout_to_bus(
    msg: ros_env::std_msgs::msg::MultiArrayLayout,
) -> crate::std_msgs::msg::v1::MultiArrayLayout {
    crate::std_msgs::msg::v1::MultiArrayLayout {
        dim: msg.dim.into_iter().map(crate::ros2_bridge::mappers::std_msgs::multi_array_dimension::multi_array_dimension_to_bus).collect(),
        data_offset: msg.data_offset.into(),
    }
}

pub(crate) fn multi_array_layout_to_ros(
    bus: crate::std_msgs::msg::v1::MultiArrayLayout,
) -> ros_env::std_msgs::msg::MultiArrayLayout {
    ros_env::std_msgs::msg::MultiArrayLayout {
        dim: bus.dim.into_iter().map(crate::ros2_bridge::mappers::std_msgs::multi_array_dimension::multi_array_dimension_to_ros).collect(),
        data_offset: bus.data_offset as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsMultiArrayLayoutMapper;

impl TypedTopicMapper for StdMsgsMultiArrayLayoutMapper {
    type Ros = ros_env::std_msgs::msg::MultiArrayLayout;
    type Bus = crate::std_msgs::msg::v1::MultiArrayLayout;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(multi_array_layout_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(multi_array_layout_to_ros(msg))
    }
}
