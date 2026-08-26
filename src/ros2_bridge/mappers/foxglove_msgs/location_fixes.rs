//! Mapper for `foxglove_msgs/msg/LocationFixes`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn location_fixes_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::LocationFixes> {
    Ok(crate::foxglove_msgs::msg::v1::LocationFixes {
        fixes: read_message_seq(view, "fixes", super::location_fix::location_fix_from_view)?,
    })
}

pub(crate) fn location_fixes_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::LocationFixes,
) -> Result<()> {
    write_message_seq(
        view,
        "fixes",
        &bus.fixes,
        super::location_fix::location_fix_write,
    )?;
    Ok(())
}

pub(crate) fn location_fixes_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::LocationFixes> {
    location_fixes_from_view(&msg.view())
}

pub(crate) fn location_fixes_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::LocationFixes,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/LocationFixes")?;
    location_fixes_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsLocationFixesMapper;
impl TopicMapper for FoxgloveMsgsLocationFixesMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LocationFixes"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(location_fixes_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::LocationFixes as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode foxglove_msgs/msg/LocationFixes: {e}"))
        })?;
        location_fixes_bus_to_dyn(&bus)
    }
}
