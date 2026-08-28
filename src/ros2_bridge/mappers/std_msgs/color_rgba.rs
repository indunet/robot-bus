//! Typed mapper for `std_msgs/msg/ColorRGBA`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn color_rgba_to_bus(msg: ros_env::std_msgs::msg::ColorRGBA) -> crate::std_msgs::msg::v1::ColorRgba {
    crate::std_msgs::msg::v1::ColorRgba {
        r: msg.r,
        g: msg.g,
        b: msg.b,
        a: msg.a,
    }
}

pub(crate) fn color_rgba_to_ros(bus: crate::std_msgs::msg::v1::ColorRgba) -> ros_env::std_msgs::msg::ColorRGBA {
    ros_env::std_msgs::msg::ColorRGBA {
        r: bus.r,
        g: bus.g,
        b: bus.b,
        a: bus.a,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsColorRgbaMapper;

impl TypedTopicMapper for StdMsgsColorRgbaMapper {
    type Ros = ros_env::std_msgs::msg::ColorRGBA;
    type Bus = crate::std_msgs::msg::v1::ColorRgba;

    fn type_name(&self) -> &'static str {
        "std_msgs/msg/ColorRGBA"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(color_rgba_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(color_rgba_to_ros(msg))
    }
}
