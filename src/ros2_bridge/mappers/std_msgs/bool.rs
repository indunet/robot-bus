//! Mapper for `std_msgs/msg/Bool`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn bool_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Bool> {
    Ok(crate::std_msgs::msg::v1::Bool {
        data: read_bool(view, "data")?,
    })
}

pub(crate) fn bool_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Bool,
) -> Result<()> {
    write_bool(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn bool_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Bool> {
    bool_from_view(&msg.view())
}

pub(crate) fn bool_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Bool,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Bool")?;
    bool_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsBoolMapper;
impl TopicMapper for StdMsgsBoolMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Bool"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(bool_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Bool as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Bool: {e}")))?;
        bool_bus_to_dyn(&bus)
    }
}
