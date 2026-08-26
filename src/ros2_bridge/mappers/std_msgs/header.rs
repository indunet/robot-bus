//! Mapper for `std_msgs/msg/Header`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn header_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Header> {
    Ok(crate::std_msgs::msg::v1::Header {
        stamp: nested_view(view, "stamp")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        frame_id: read_string(view, "frame_id")?,
    })
}

pub(crate) fn header_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Header,
) -> Result<()> {
    if let Some(v) = &bus.stamp {
        with_nested_mut(view, "stamp", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    Ok(())
}

pub(crate) fn header_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Header> {
    header_from_view(&msg.view())
}

pub(crate) fn header_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Header,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Header")?;
    header_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsHeaderMapper;
impl TopicMapper for StdMsgsHeaderMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Header"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(header_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Header as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Header: {e}")))?;
        header_bus_to_dyn(&bus)
    }
}
