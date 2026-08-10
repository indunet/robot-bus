//! Mapper for `std_msgs/msg/Int32MultiArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn int32_multi_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Int32MultiArray> {
    Ok(crate::std_msgs::msg::v1::Int32MultiArray {
        layout: nested_view(view, "layout")?
            .as_ref()
            .map(super::multi_array_layout::multi_array_layout_from_view)
            .transpose()?,
        data: read_i32_seq(view, "data")?,
    })
}

pub(crate) fn int32_multi_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Int32MultiArray,
) -> Result<()> {
    if let Some(v) = &bus.layout {
        with_nested_mut(view, "layout", |nested| super::multi_array_layout::multi_array_layout_write(nested, v))?;
    }
    write_i32_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn int32_multi_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Int32MultiArray> {
    int32_multi_array_from_view(&msg.view())
}

pub(crate) fn int32_multi_array_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Int32MultiArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Int32MultiArray")?;
    int32_multi_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsInt32MultiArrayMapper;
impl TopicMapper for StdMsgsInt32MultiArrayMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Int32MultiArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(int32_multi_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Int32MultiArray as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Int32MultiArray: {e}")))?;
        int32_multi_array_bus_to_dyn(&bus)
    }
}
