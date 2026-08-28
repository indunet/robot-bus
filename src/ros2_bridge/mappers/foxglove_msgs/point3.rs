//! Typed mapper for `foxglove_msgs/msg/Point3`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point3_to_bus(msg: ros_env::foxglove_msgs::msg::Point3) -> crate::foxglove_msgs::msg::v1::Point3 {
    crate::foxglove_msgs::msg::v1::Point3 {
        x: msg.x,
        y: msg.y,
        z: msg.z,
    }
}

pub(crate) fn point3_to_ros(bus: crate::foxglove_msgs::msg::v1::Point3) -> ros_env::foxglove_msgs::msg::Point3 {
    ros_env::foxglove_msgs::msg::Point3 {
        x: bus.x,
        y: bus.y,
        z: bus.z,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPoint3Mapper;

impl TypedTopicMapper for FoxgloveMsgsPoint3Mapper {
    type Ros = ros_env::foxglove_msgs::msg::Point3;
    type Bus = crate::foxglove_msgs::msg::v1::Point3;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Point3"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point3_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point3_to_ros(msg))
    }
}
