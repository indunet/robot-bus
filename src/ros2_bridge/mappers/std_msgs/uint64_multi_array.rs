//! Typed mapper for `std_msgs/msg/UInt64MultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uint64_multi_array_to_bus(msg: ros_env::std_msgs::msg::UInt64MultiArray) -> crate::std_msgs::msg::v1::UInt64MultiArray {
    crate::std_msgs::msg::v1::UInt64MultiArray {
        layout: Some(crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(msg.layout)),
        data: msg.data.into_iter().collect(),
    }
}

pub(crate) fn uint64_multi_array_to_ros(bus: crate::std_msgs::msg::v1::UInt64MultiArray) -> ros_env::std_msgs::msg::UInt64MultiArray {
    ros_env::std_msgs::msg::UInt64MultiArray {
        layout: crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(bus.layout.unwrap_or_default()),
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsUInt64MultiArrayMapper;

impl TypedTopicMapper for StdMsgsUInt64MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::UInt64MultiArray;
    type Bus = crate::std_msgs::msg::v1::UInt64MultiArray;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/UInt64MultiArray"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uint64_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uint64_multi_array_to_ros(msg))
    }
}
