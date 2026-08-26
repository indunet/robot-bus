//! Mapper for `builtin_interfaces/msg/Time`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn time_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::builtin_interfaces::msg::v1::Time> {
    Ok(crate::builtin_interfaces::msg::v1::Time {
        sec: read_i32(view, "sec")?,
        nanosec: read_u32(view, "nanosec")?,
    })
}

pub(crate) fn time_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::builtin_interfaces::msg::v1::Time,
) -> Result<()> {
    write_i32(view, "sec", bus.sec)?;
    write_u32(view, "nanosec", bus.nanosec)?;
    Ok(())
}

pub(crate) fn time_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::builtin_interfaces::msg::v1::Time> {
    time_from_view(&msg.view())
}

pub(crate) fn time_bus_to_dyn(
    bus: &crate::builtin_interfaces::msg::v1::Time,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("builtin_interfaces/msg/Time")?;
    time_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct BuiltinInterfacesTimeMapper;
impl TopicMapper for BuiltinInterfacesTimeMapper {
    fn type_name(&self) -> &'static str {
        "builtin_interfaces/msg/Time"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(time_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::builtin_interfaces::msg::v1::Time as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode builtin_interfaces/msg/Time: {e}")))?;
        time_bus_to_dyn(&bus)
    }
}
