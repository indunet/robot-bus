//! Typed mapper for `std_msgs/msg/Int8MultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn int8_multi_array_to_bus(msg: ros_env::std_msgs::msg::Int8MultiArray) -> crate::std_msgs::msg::v1::Int8MultiArray {
    crate::std_msgs::msg::v1::Int8MultiArray {
        layout: Some(crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(msg.layout)),
        data: crate::ros2_bridge::mappers::convert::i32_seq(msg.data),
    }
}

pub(crate) fn int8_multi_array_to_ros(bus: crate::std_msgs::msg::v1::Int8MultiArray) -> ros_env::std_msgs::msg::Int8MultiArray {
    ros_env::std_msgs::msg::Int8MultiArray {
        layout: crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(bus.layout.unwrap_or_default()),
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt8MultiArrayMapper;

impl TypedTopicMapper for StdMsgsInt8MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::Int8MultiArray;
    type Bus = crate::std_msgs::msg::v1::Int8MultiArray;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int8MultiArray"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(int8_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(int8_multi_array_to_ros(msg))
    }
}
