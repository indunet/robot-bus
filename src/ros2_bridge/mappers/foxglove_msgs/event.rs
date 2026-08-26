//! Mapper for `foxglove_msgs/msg/Event`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn event_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Event> {
    Ok(crate::foxglove_msgs::msg::v1::Event {
        start_time: read_timestamp(view, "start_time")?,
        end_time: read_timestamp(view, "end_time")?,
        metadata: read_message_seq(
            view,
            "metadata",
            super::key_value_pair::key_value_pair_from_view,
        )?,
    })
}

pub(crate) fn event_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Event,
) -> Result<()> {
    if let Some(v) = &bus.start_time {
        write_timestamp(view, "start_time", v)?;
    }
    if let Some(v) = &bus.end_time {
        write_timestamp(view, "end_time", v)?;
    }
    write_message_seq(
        view,
        "metadata",
        &bus.metadata,
        super::key_value_pair::key_value_pair_write,
    )?;
    Ok(())
}

pub(crate) fn event_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Event> {
    event_from_view(&msg.view())
}

pub(crate) fn event_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Event,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Event")?;
    event_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsEventMapper;
impl TopicMapper for FoxgloveMsgsEventMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Event"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(event_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Event as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Event: {e}")))?;
        event_bus_to_dyn(&bus)
    }
}
