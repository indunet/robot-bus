//! Mapper for `shape_msgs/msg/Plane`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn plane_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::shape_msgs::msg::v1::Plane> {
    Ok(crate::shape_msgs::msg::v1::Plane {
        coef: read_f64_seq(view, "coef")?,
    })
}

pub(crate) fn plane_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::shape_msgs::msg::v1::Plane,
) -> Result<()> {
    write_f64_seq(view, "coef", &bus.coef)?;
    Ok(())
}

pub(crate) fn plane_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::shape_msgs::msg::v1::Plane> {
    plane_from_view(&msg.view())
}

pub(crate) fn plane_bus_to_dyn(
    bus: &crate::shape_msgs::msg::v1::Plane,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("shape_msgs/msg/Plane")?;
    plane_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ShapeMsgsPlaneMapper;
impl TopicMapper for ShapeMsgsPlaneMapper {
    fn type_name(&self) -> &'static str {
        "shape_msgs/msg/Plane"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(plane_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::shape_msgs::msg::v1::Plane as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode shape_msgs/msg/Plane: {e}")))?;
        plane_bus_to_dyn(&bus)
    }
}
