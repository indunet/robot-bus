//! Typed mapper for `std_msgs/msg/ByteMultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn byte_multi_array_to_bus(
    msg: ros_env::std_msgs::msg::ByteMultiArray,
) -> crate::std_msgs::msg::v1::ByteMultiArray {
    crate::std_msgs::msg::v1::ByteMultiArray {
        layout: Some(
            crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(
                msg.layout,
            ),
        ),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn byte_multi_array_to_ros(
    bus: crate::std_msgs::msg::v1::ByteMultiArray,
) -> ros_env::std_msgs::msg::ByteMultiArray {
    ros_env::std_msgs::msg::ByteMultiArray {
        layout:
            crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(
                bus.layout.unwrap_or_default(),
            ),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsByteMultiArrayMapper;

impl TypedTopicMapper for StdMsgsByteMultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::ByteMultiArray;
    type Bus = crate::std_msgs::msg::v1::ByteMultiArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(byte_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(byte_multi_array_to_ros(msg))
    }
}
