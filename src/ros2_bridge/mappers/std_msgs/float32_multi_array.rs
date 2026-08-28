//! Typed mapper for `std_msgs/msg/Float32MultiArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn float32_multi_array_to_bus(msg: ros_env::std_msgs::msg::Float32MultiArray) -> crate::std_msgs::msg::v1::Float32MultiArray {
    crate::std_msgs::msg::v1::Float32MultiArray {
        layout: Some(crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_bus(msg.layout)),
        data: crate::ros2_bridge::mappers::convert::f32_seq(msg.data),
    }
}

pub(crate) fn float32_multi_array_to_ros(bus: crate::std_msgs::msg::v1::Float32MultiArray) -> ros_env::std_msgs::msg::Float32MultiArray {
    ros_env::std_msgs::msg::Float32MultiArray {
        layout: crate::ros2_bridge::mappers::std_msgs::multi_array_layout::multi_array_layout_to_ros(bus.layout.unwrap_or_default()),
        data: bus.data,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsFloat32MultiArrayMapper;

impl TypedTopicMapper for StdMsgsFloat32MultiArrayMapper {
    type Ros = ros_env::std_msgs::msg::Float32MultiArray;
    type Bus = crate::std_msgs::msg::v1::Float32MultiArray;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Float32MultiArray"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(float32_multi_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(float32_multi_array_to_ros(msg))
    }
}
