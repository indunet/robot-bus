//! Typed mapper for `std_msgs/msg/ByteMultiArray`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn byte_multi_array_to_bus(
    msg: ros_env::std_msgs::msg::ByteMultiArray,
) -> robot_bus::std_msgs::msg::v1::ByteMultiArray {
    robot_bus::std_msgs::msg::v1::ByteMultiArray {
        layout: Some(
            crate::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(
                msg.layout,
            ),
        ),
        data: crate::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn byte_multi_array_to_ros(
    bus: robot_bus::std_msgs::msg::v1::ByteMultiArray,
) -> ros_env::std_msgs::msg::ByteMultiArray {
    ros_env::std_msgs::msg::ByteMultiArray {
        layout:
            crate::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(
                bus.layout.unwrap_or_default(),
            ),
        data: crate::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsByteMultiArrayMapper;

impl TypedTopicMapper for StdMsgsByteMultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::ByteMultiArray;
    type Bus = robot_bus::std_msgs::msg::v1::ByteMultiArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(byte_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(byte_multi_array_to_ros(msg))
    }
}
