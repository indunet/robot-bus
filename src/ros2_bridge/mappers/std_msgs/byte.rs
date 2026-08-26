//! Mapper for `std_msgs/msg/Byte`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn byte_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Byte> {
    Ok(crate::std_msgs::msg::v1::Byte {
        data: read_u32(view, "data")?,
    })
}

pub(crate) fn byte_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Byte,
) -> Result<()> {
    write_u32(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn byte_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Byte> {
    byte_from_view(&msg.view())
}

pub(crate) fn byte_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Byte,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Byte")?;
    byte_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsByteMapper;
impl TopicMapper for StdMsgsByteMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Byte"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(byte_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Byte as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Byte: {e}")))?;
        byte_bus_to_dyn(&bus)
    }
}
