//! Typed mapper for `std_msgs/msg/Int16MultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn int16_multi_array_to_bus(msg: ros_env::std_msgs::msg::Int16MultiArray) -> crate::std_msgs::msg::v1::Int16MultiArray {
    crate::std_msgs::msg::v1::Int16MultiArray {
        layout: Some(crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(msg.layout)),
        data: crate::ros2_bridge::mappers::convert::i32_seq(msg.data),
    }
}

pub(crate) fn int16_multi_array_to_ros(bus: crate::std_msgs::msg::v1::Int16MultiArray) -> ros_env::std_msgs::msg::Int16MultiArray {
    ros_env::std_msgs::msg::Int16MultiArray {
        layout: crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(bus.layout.unwrap_or_default()),
        data: crate::ros2_bridge::mappers::convert::FromI32Seq::from_i32_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt16MultiArrayMapper;

impl TypedTopicMapper for StdMsgsInt16MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::Int16MultiArray;
    type Bus = crate::std_msgs::msg::v1::Int16MultiArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(int16_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(int16_multi_array_to_ros(msg))
    }
}
