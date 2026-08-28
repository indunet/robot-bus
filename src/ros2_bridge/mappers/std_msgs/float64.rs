//! Typed mapper for `std_msgs/msg/Float64`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn float64_to_bus(msg: ros_env::std_msgs::msg::Float64) -> crate::std_msgs::msg::v1::Float64 {
    crate::std_msgs::msg::v1::Float64 {
        data: msg.data,
    }
}

pub(crate) fn float64_to_ros(bus: crate::std_msgs::msg::v1::Float64) -> ros_env::std_msgs::msg::Float64 {
    ros_env::std_msgs::msg::Float64 {
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsFloat64Mapper;

impl TypedTopicMapper for StdMsgsFloat64Mapper {
    type Ros = ros_env::std_msgs::msg::Float64;
    type Bus = crate::std_msgs::msg::v1::Float64;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Float64"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(float64_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(float64_to_ros(msg))
    }
}
