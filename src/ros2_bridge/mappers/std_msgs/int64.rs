//! Typed mapper for `std_msgs/msg/Int64`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn int64_to_bus(msg: ros_env::std_msgs::msg::Int64) -> crate::std_msgs::msg::v1::Int64 {
    crate::std_msgs::msg::v1::Int64 {
        data: msg.data,
    }
}

pub(crate) fn int64_to_ros(bus: crate::std_msgs::msg::v1::Int64) -> ros_env::std_msgs::msg::Int64 {
    ros_env::std_msgs::msg::Int64 {
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsInt64Mapper;

impl TypedTopicMapper for StdMsgsInt64Mapper {
    type Ros = ros_env::std_msgs::msg::Int64;
    type Bus = crate::std_msgs::msg::v1::Int64;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int64"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(int64_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(int64_to_ros(msg))
    }
}
