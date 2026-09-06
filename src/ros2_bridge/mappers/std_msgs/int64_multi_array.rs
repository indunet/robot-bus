//! Typed mapper for `std_msgs/msg/Int64MultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn int64_multi_array_to_bus(
    msg: ros_env::std_msgs::msg::Int64MultiArray,
) -> crate::std_msgs::msg::v1::Int64MultiArray {
    crate::std_msgs::msg::v1::Int64MultiArray {
        layout: Some(
            crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(
                msg.layout,
            ),
        ),
        data: crate::ros2_bridge::mappers::convert::i64_seq(msg.data),
    }
}

pub(crate) fn int64_multi_array_to_ros(
    bus: crate::std_msgs::msg::v1::Int64MultiArray,
) -> ros_env::std_msgs::msg::Int64MultiArray {
    ros_env::std_msgs::msg::Int64MultiArray {
        layout:
            crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(
                bus.layout.unwrap_or_default(),
            ),
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt64MultiArrayMapper;

impl TypedTopicMapper for StdMsgsInt64MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::Int64MultiArray;
    type Bus = crate::std_msgs::msg::v1::Int64MultiArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(int64_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(int64_multi_array_to_ros(msg))
    }
}
