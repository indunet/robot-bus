//! Typed mapper for `foxglove_msgs/msg/PackedElementField`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn packed_element_field_to_bus(msg: ros_env::foxglove_msgs::msg::PackedElementField) -> crate::foxglove_msgs::msg::v1::PackedElementField {
    crate::foxglove_msgs::msg::v1::PackedElementField {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        offset: msg.offset,
        r#type: msg.type_ as i32,
    }
}

pub(crate) fn packed_element_field_to_ros(bus: crate::foxglove_msgs::msg::v1::PackedElementField) -> ros_env::foxglove_msgs::msg::PackedElementField {
    ros_env::foxglove_msgs::msg::PackedElementField {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        offset: bus.offset,
        type_: bus.r#type as i32,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsPackedElementFieldMapper;

impl TypedTopicMapper for FoxgloveMsgsPackedElementFieldMapper {
    type Ros = ros_env::foxglove_msgs::msg::PackedElementField;
    type Bus = crate::foxglove_msgs::msg::v1::PackedElementField;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PackedElementField"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(packed_element_field_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(packed_element_field_to_ros(msg))
    }
}
