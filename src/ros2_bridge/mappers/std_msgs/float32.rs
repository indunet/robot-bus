//! Mapper for `std_msgs/msg/Float32`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn float32_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::std_msgs::msg::v1::Float32> {
    Ok(crate::std_msgs::msg::v1::Float32 {
        data: read_f32(view, "data")?,
    })
}

pub(crate) fn float32_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::std_msgs::msg::v1::Float32,
) -> Result<()> {
    write_f32(view, "data", bus.data)?;
    Ok(())
}

pub(crate) fn float32_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::std_msgs::msg::v1::Float32> {
    float32_from_view(&msg.view())
}

pub(crate) fn float32_bus_to_dyn(
    bus: &crate::std_msgs::msg::v1::Float32,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("std_msgs/msg/Float32")?;
    float32_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StdMsgsFloat32Mapper;
impl TopicMapper for StdMsgsFloat32Mapper {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/Float32"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(float32_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::std_msgs::msg::v1::Float32 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode std_msgs/msg/Float32: {e}")))?;
        float32_bus_to_dyn(&bus)
    }
}
