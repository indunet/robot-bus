//! Typed mapper for `shape_msgs/msg/SolidPrimitive`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn solid_primitive_to_bus(
    msg: ros_env::shape_msgs::msg::SolidPrimitive,
) -> robot_bus::shape_msgs::msg::v1::SolidPrimitive {
    robot_bus::shape_msgs::msg::v1::SolidPrimitive {
        r#type: msg.type_.into(),
        dimensions: crate::mappers::convert::f64_seq(msg.dimensions),
        polygon: Some(
            crate::mappers::geometry_msgs::polygon::polygon_to_bus(msg.polygon),
        ),
    }
}

pub(crate) fn solid_primitive_to_ros(
    bus: robot_bus::shape_msgs::msg::v1::SolidPrimitive,
) -> ros_env::shape_msgs::msg::SolidPrimitive {
    ros_env::shape_msgs::msg::SolidPrimitive {
        type_: bus.r#type as _,
        dimensions: crate::mappers::convert::FromF64Seq::from_f64_seq(bus.dimensions),
        polygon: crate::mappers::geometry_msgs::polygon::polygon_to_ros(
            bus.polygon.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeMsgsSolidPrimitiveMapper;

impl TypedTopicMapper for ShapeMsgsSolidPrimitiveMapper {
    type Ros = ros_env::shape_msgs::msg::SolidPrimitive;
    type Bus = robot_bus::shape_msgs::msg::v1::SolidPrimitive;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(solid_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(solid_primitive_to_ros(msg))
    }
}
