//! Mapper for `std_msgs/msg/Float64MultiArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn float64_multi_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Float64MultiArray> {
    Ok(crate::std_msgs::msg::v1::Float64MultiArray {
        layout: nested_view(view, "layout")?
            .as_ref()
            .map(super::multi_array_layout::multi_array_layout_from_view)
            .transpose()?,
        data: read_f64_seq(view, "data")?,
    })
}

pub(crate) fn float64_multi_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Float64MultiArray,
) -> Result<()> {
    if let Some(v) = &bus.layout {
        with_nested_mut(view, "layout", |nested| {
            super::multi_array_layout::multi_array_layout_write(nested, v)
        })?;
    }
    write_f64_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn float64_multi_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Float64MultiArray> {
    float64_multi_array_from_view(&msg.view())
}

pub(crate) fn float64_multi_array_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Float64MultiArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Float64MultiArray")?;
    float64_multi_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsFloat64MultiArrayMapper;
impl TopicMapper for StdMsgsFloat64MultiArrayMapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Float64MultiArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(float64_multi_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Float64MultiArray as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode std_msgs/msg/Float64MultiArray: {e}"))
            })?;
        float64_multi_array_bus_to_dyn(&bus)
    }
}
