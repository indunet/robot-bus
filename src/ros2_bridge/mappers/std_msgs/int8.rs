//! Mapper for `std_msgs/msg/Int8`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn int8_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Int8> {
    Ok(crate::std_msgs::msg::v1::Int8 {
        data: read_i32(view, "data")?,
    })
}

pub(crate) fn int8_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Int8,
) -> Result<()> {
    write_i32(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn int8_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Int8> {
    int8_from_view(&msg.view())
}

pub(crate) fn int8_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Int8,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Int8")?;
    int8_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsInt8Mapper;
impl TopicMapper for StdMsgsInt8Mapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int8"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(int8_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Int8 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Int8: {e}")))?;
        int8_bus_to_dyn(&bus)
    }
}
