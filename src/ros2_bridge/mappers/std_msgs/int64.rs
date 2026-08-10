//! Mapper for `std_msgs/msg/Int64`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn int64_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Int64> {
    Ok(crate::std_msgs::msg::v1::Int64 {
        data: read_i64(view, "data")?,
    })
}

pub(crate) fn int64_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Int64,
) -> Result<()> {
    write_i64(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn int64_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Int64> {
    int64_from_view(&msg.view())
}

pub(crate) fn int64_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Int64,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Int64")?;
    int64_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsInt64Mapper;
impl TopicMapper for StdMsgsInt64Mapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int64"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(int64_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Int64 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Int64: {e}")))?;
        int64_bus_to_dyn(&bus)
    }
}
