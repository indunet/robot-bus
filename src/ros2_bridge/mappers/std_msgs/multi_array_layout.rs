//! Mapper for `std_msgs/msg/MultiArrayLayout`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn multi_array_layout_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::MultiArrayLayout> {
    Ok(crate::std_msgs::msg::v1::MultiArrayLayout {
        dim: read_message_seq(
            view,
            "dim",
            super::multi_array_dimension::multi_array_dimension_from_view,
        )?,
        data_offset: read_u32(view, "data_offset")?,
    })
}

pub(crate) fn multi_array_layout_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::MultiArrayLayout,
) -> Result<()> {
    write_message_seq(
        view,
        "dim",
        &bus.dim,
        super::multi_array_dimension::multi_array_dimension_write,
    )?;
    write_u32(view, "data_offset", bus.data_offset)?;
    Ok(())
}

pub(crate) fn multi_array_layout_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::MultiArrayLayout> {
    multi_array_layout_from_view(&msg.view())
}

pub(crate) fn multi_array_layout_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::MultiArrayLayout,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/MultiArrayLayout")?;
    multi_array_layout_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsMultiArrayLayoutMapper;
impl TopicMapper for StdMsgsMultiArrayLayoutMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/MultiArrayLayout"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_array_layout_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::MultiArrayLayout as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode std_msgs/msg/MultiArrayLayout: {e}"))
            })?;
        multi_array_layout_bus_to_dyn(&bus)
    }
}
