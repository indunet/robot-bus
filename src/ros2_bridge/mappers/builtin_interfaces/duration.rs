//! Mapper for `builtin_interfaces/msg/Duration`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn duration_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::builtin_interfaces::msg::v1::Duration> {
    Ok(crate::builtin_interfaces::msg::v1::Duration {
        sec: read_i32(view, "sec")?,
        nanosec: read_u32(view, "nanosec")?,
    })
}

pub(crate) fn duration_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::builtin_interfaces::msg::v1::Duration,
) -> Result<()> {
    write_i32(view, "sec", bus.sec)?;
    write_u32(view, "nanosec", bus.nanosec)?;
    Ok(())
}

pub(crate) fn duration_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::builtin_interfaces::msg::v1::Duration> {
    duration_from_view(&msg.view())
}

pub(crate) fn duration_bus_to_dyn(
    bus: &crate::builtin_interfaces::msg::v1::Duration,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("builtin_interfaces/msg/Duration")?;
    duration_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct BuiltinInterfacesDurationMapper;
impl TopicMapper for BuiltinInterfacesDurationMapper {
    fn type_name(&self) -> &'static str {
        "builtin_interfaces/msg/Duration"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(duration_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::builtin_interfaces::msg::v1::Duration as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode builtin_interfaces/msg/Duration: {e}"))
        })?;
        duration_bus_to_dyn(&bus)
    }
}
