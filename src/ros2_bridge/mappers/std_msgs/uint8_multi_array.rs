//! Typed mapper for `std_msgs/msg/UInt8MultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uint8_multi_array_to_bus(
    msg: ros_env::std_msgs::msg::UInt8MultiArray,
) -> crate::std_msgs::msg::v1::UInt8MultiArray {
    crate::std_msgs::msg::v1::UInt8MultiArray {
        layout: Some(
            crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(
                msg.layout,
            ),
        ),
        data: crate::ros2_bridge::mappers::convert::u32_seq(msg.data),
    }
}

pub(crate) fn uint8_multi_array_to_ros(
    bus: crate::std_msgs::msg::v1::UInt8MultiArray,
) -> ros_env::std_msgs::msg::UInt8MultiArray {
    ros_env::std_msgs::msg::UInt8MultiArray {
        layout:
            crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(
                bus.layout.unwrap_or_default(),
            ),
        data: crate::ros2_bridge::mappers::convert::FromU32Seq::from_u32_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt8MultiArrayMapper;

impl TypedTopicMapper for StdMsgsUInt8MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::UInt8MultiArray;
    type Bus = crate::std_msgs::msg::v1::UInt8MultiArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uint8_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uint8_multi_array_to_ros(msg))
    }
}
