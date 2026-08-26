//! Mapper for `foxglove_msgs/msg/Point3`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point3_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Point3> {
    Ok(crate::foxglove_msgs::msg::v1::Point3 {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
        z: read_f64(view, "z")?,
    })
}

pub(crate) fn point3_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Point3,
) -> Result<()> {
    write_f64(view, "x", bus.x)?;
    write_f64(view, "y", bus.y)?;
    write_f64(view, "z", bus.z)?;
    Ok(())
}

pub(crate) fn point3_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Point3> {
    point3_from_view(&msg.view())
}

pub(crate) fn point3_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Point3,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Point3")?;
    point3_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPoint3Mapper;
impl TopicMapper for FoxgloveMsgsPoint3Mapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Point3"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point3_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Point3 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Point3: {e}")))?;
        point3_bus_to_dyn(&bus)
    }
}
