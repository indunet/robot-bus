//! Mapper for `std_msgs/msg/String`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn string_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::String> {
    Ok(crate::std_msgs::msg::v1::String {
        data: read_string(view, "data")?,
    })
}

pub(crate) fn string_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::String,
) -> Result<()> {
    write_string(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn string_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::String> {
    string_from_view(&msg.view())
}

pub(crate) fn string_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::String,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/String")?;
    string_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsStringMapper;
impl TopicMapper for StdMsgsStringMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/String"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(string_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::String as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/String: {e}")))?;
        string_bus_to_dyn(&bus)
    }
}
