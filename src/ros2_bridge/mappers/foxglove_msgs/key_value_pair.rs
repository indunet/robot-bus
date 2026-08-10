//! Mapper for `foxglove_msgs/msg/KeyValuePair`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn key_value_pair_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::KeyValuePair> {
    Ok(crate::foxglove_msgs::msg::v1::KeyValuePair {
        key: read_string(view, "key")?,
        value: read_string(view, "value")?,
    })
}

pub(crate) fn key_value_pair_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::KeyValuePair,
) -> Result<()> {
    write_string(view, "key", &bus.key)?;
    write_string(view, "value", &bus.value)?;
    Ok(())
}

pub(crate) fn key_value_pair_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::KeyValuePair> {
    key_value_pair_from_view(&msg.view())
}

pub(crate) fn key_value_pair_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::KeyValuePair,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/KeyValuePair")?;
    key_value_pair_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsKeyValuePairMapper;
impl TopicMapper for FoxgloveMsgsKeyValuePairMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/KeyValuePair"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(key_value_pair_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::KeyValuePair as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/KeyValuePair: {e}"))
            })?;
        key_value_pair_bus_to_dyn(&bus)
    }
}
