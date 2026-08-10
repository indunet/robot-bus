//! Mapper for `foxglove_msgs/msg/Quaternion`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn quaternion_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::Quaternion> {
    Ok(crate::foxglove_msgs::msg::v1::Quaternion {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
        z: read_f64(view, "z")?,
        w: read_f64(view, "w")?,
    })
}

pub(crate) fn quaternion_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::Quaternion,
) -> Result<()> {
    write_f64(view, "x", bus.x)?;
    write_f64(view, "y", bus.y)?;
    write_f64(view, "z", bus.z)?;
    write_f64(view, "w", bus.w)?;
    Ok(())
}

pub(crate) fn quaternion_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::Quaternion> {
    quaternion_from_view(&msg.view())
}

pub(crate) fn quaternion_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::Quaternion,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/Quaternion")?;
    quaternion_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsQuaternionMapper;
impl TopicMapper for FoxgloveMsgsQuaternionMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/Quaternion"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(quaternion_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::Quaternion as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/Quaternion: {e}")))?;
        quaternion_bus_to_dyn(&bus)
    }
}
