//! Typed mapper for `shape_msgs/msg/SolidPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn solid_primitive_to_bus(msg: ros_env::shape_msgs::msg::SolidPrimitive) -> crate::shape_msgs::msg::v1::SolidPrimitive {
    crate::shape_msgs::msg::v1::SolidPrimitive {
        r#type: msg.type_.into(),
        dimensions: crate::ros2_bridge::mappers::convert::f64_seq(msg.dimensions),
        polygon: Some(crate::ros2_bridge::mappers::geometry_msgs::polygon::polygon_to_bus(msg.polygon)),
    }
}

pub(crate) fn solid_primitive_to_ros(bus: crate::shape_msgs::msg::v1::SolidPrimitive) -> ros_env::shape_msgs::msg::SolidPrimitive {
    ros_env::shape_msgs::msg::SolidPrimitive {
        type_: bus.r#type as _,
        dimensions: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.dimensions),
        polygon: crate::ros2_bridge::mappers::geometry_msgs::polygon::polygon_to_ros(bus.polygon.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeMsgsSolidPrimitiveMapper;

impl TypedTopicMapper for ShapeMsgsSolidPrimitiveMapper {
    type Ros = ros_env::shape_msgs::msg::SolidPrimitive;
    type Bus = crate::shape_msgs::msg::v1::SolidPrimitive;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(solid_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(solid_primitive_to_ros(msg))
    }
}
