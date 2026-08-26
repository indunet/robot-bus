//! Mapper for `std_msgs/msg/Empty`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn empty_from_view(
    _view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Empty> {
    Ok(crate::std_msgs::msg::v1::Empty {})
}

pub(crate) fn empty_write(
    _view: &mut rclrs::DynamicMessageViewMut<'_>,
    _bus: &crate::std_msgs::msg::v1::Empty,
) -> Result<()> {
    Ok(())
}

pub(crate) fn empty_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Empty> {
    empty_from_view(&msg.view())
}

pub(crate) fn empty_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Empty,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Empty")?;
    empty_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsEmptyMapper;
impl TopicMapper for StdMsgsEmptyMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Empty"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(empty_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Empty as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Empty: {e}")))?;
        empty_bus_to_dyn(&bus)
    }
}
