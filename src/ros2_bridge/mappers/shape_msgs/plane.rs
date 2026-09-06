//! Typed mapper for `shape_msgs/msg/Plane`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn plane_to_bus(
    msg: ros_env::shape_msgs::msg::Plane,
) -> crate::shape_msgs::msg::v1::Plane {
    crate::shape_msgs::msg::v1::Plane {
        coef: crate::ros2_bridge::mappers::convert::f64_seq(msg.coef),
    }
}

pub(crate) fn plane_to_ros(
    bus: crate::shape_msgs::msg::v1::Plane,
) -> ros_env::shape_msgs::msg::Plane {
    ros_env::shape_msgs::msg::Plane {
        coef: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.coef),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeMsgsPlaneMapper;

impl TypedTopicMapper for ShapeMsgsPlaneMapper {
    type Ros = ros_env::shape_msgs::msg::Plane;
    type Bus = crate::shape_msgs::msg::v1::Plane;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(plane_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(plane_to_ros(msg))
    }
}
