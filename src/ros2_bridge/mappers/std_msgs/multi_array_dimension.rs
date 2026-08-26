//! Mapper for `std_msgs/msg/MultiArrayDimension`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn multi_array_dimension_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::MultiArrayDimension> {
    Ok(crate::std_msgs::msg::v1::MultiArrayDimension {
        label: read_string(view, "label")?,
        size: read_u32(view, "size")?,
        stride: read_u32(view, "stride")?,
    })
}

pub(crate) fn multi_array_dimension_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::MultiArrayDimension,
) -> Result<()> {
    write_string(view, "label", &bus.label)?;
    write_u32(view, "size", bus.size)?;
    write_u32(view, "stride", bus.stride)?;
    Ok(())
}

pub(crate) fn multi_array_dimension_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::MultiArrayDimension> {
    multi_array_dimension_from_view(&msg.view())
}

pub(crate) fn multi_array_dimension_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::MultiArrayDimension,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/MultiArrayDimension")?;
    multi_array_dimension_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsMultiArrayDimensionMapper;
impl TopicMapper for StdMsgsMultiArrayDimensionMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/MultiArrayDimension"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_array_dimension_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::MultiArrayDimension as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode std_msgs/msg/MultiArrayDimension: {e}"))
            })?;
        multi_array_dimension_bus_to_dyn(&bus)
    }
}
