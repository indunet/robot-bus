//! Mapper for `std_msgs/msg/Int32`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn int32_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Int32> {
    Ok(crate::std_msgs::msg::v1::Int32 {
        data: read_i32(view, "data")?,
    })
}

pub(crate) fn int32_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Int32,
) -> Result<()> {
    write_i32(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn int32_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Int32> {
    int32_from_view(&msg.view())
}

pub(crate) fn int32_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Int32,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Int32")?;
    int32_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsInt32Mapper;
impl TopicMapper for StdMsgsInt32Mapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int32"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(int32_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Int32 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Int32: {e}")))?;
        int32_bus_to_dyn(&bus)
    }
}
