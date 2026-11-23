//! Typed mapper for `std_msgs/msg/UInt32MultiArray`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn uint32_multi_array_to_bus(
    msg: ros_env::std_msgs::msg::UInt32MultiArray,
) -> robot_bus::std_msgs::msg::v1::UInt32MultiArray {
    robot_bus::std_msgs::msg::v1::UInt32MultiArray {
        layout: Some(
            crate::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(
                msg.layout,
            ),
        ),
        data: crate::mappers::convert::u32_seq(msg.data),
    }
}

pub(crate) fn uint32_multi_array_to_ros(
    bus: robot_bus::std_msgs::msg::v1::UInt32MultiArray,
) -> ros_env::std_msgs::msg::UInt32MultiArray {
    ros_env::std_msgs::msg::UInt32MultiArray {
        layout:
            crate::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(
                bus.layout.unwrap_or_default(),
            ),
        data: crate::mappers::convert::FromU32Seq::from_u32_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt32MultiArrayMapper;

impl TypedTopicMapper for StdMsgsUInt32MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::UInt32MultiArray;
    type Bus = robot_bus::std_msgs::msg::v1::UInt32MultiArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(uint32_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(uint32_multi_array_to_ros(msg))
    }
}
