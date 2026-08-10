//! Mapper for `std_msgs/msg/UInt8`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn u_int8_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::UInt8> {
    Ok(crate::std_msgs::msg::v1::UInt8 {
        data: read_u32(view, "data")?,
    })
}

pub(crate) fn u_int8_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::UInt8,
) -> Result<()> {
    write_u32(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn u_int8_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::UInt8> {
    u_int8_from_view(&msg.view())
}

pub(crate) fn u_int8_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::UInt8,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/UInt8")?;
    u_int8_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsUInt8Mapper;
impl TopicMapper for StdMsgsUInt8Mapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/UInt8"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(u_int8_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::UInt8 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/UInt8: {e}")))?;
        u_int8_bus_to_dyn(&bus)
    }
}
