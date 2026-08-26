//! Mapper for `std_msgs/msg/UInt16`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn u_int16_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::UInt16> {
    Ok(crate::std_msgs::msg::v1::UInt16 {
        data: read_u32(view, "data")?,
    })
}

pub(crate) fn u_int16_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::UInt16,
) -> Result<()> {
    write_u32(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn u_int16_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::UInt16> {
    u_int16_from_view(&msg.view())
}

pub(crate) fn u_int16_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::UInt16,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/UInt16")?;
    u_int16_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsUInt16Mapper;
impl TopicMapper for StdMsgsUInt16Mapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/UInt16"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(u_int16_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::UInt16 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/UInt16: {e}")))?;
        u_int16_bus_to_dyn(&bus)
    }
}
