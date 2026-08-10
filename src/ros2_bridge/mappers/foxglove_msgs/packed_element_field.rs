//! Mapper for `foxglove_msgs/msg/PackedElementField`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn packed_element_field_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::PackedElementField> {
    Ok(crate::foxglove_msgs::msg::v1::PackedElementField {
        name: read_string(view, "name")?,
        offset: read_u32(view, "offset")?,
        r#type: read_i32(view, "type")?,
    })
}

pub(crate) fn packed_element_field_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::PackedElementField,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    write_u32(view, "offset", bus.offset)?;
    write_i32(view, "type", bus.r#type)?;
    Ok(())
}

pub(crate) fn packed_element_field_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::PackedElementField> {
    packed_element_field_from_view(&msg.view())
}

pub(crate) fn packed_element_field_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::PackedElementField,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/PackedElementField")?;
    packed_element_field_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPackedElementFieldMapper;
impl TopicMapper for FoxgloveMsgsPackedElementFieldMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PackedElementField"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(packed_element_field_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::PackedElementField as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode foxglove_msgs/msg/PackedElementField: {e}"))
                })?;
        packed_element_field_bus_to_dyn(&bus)
    }
}
