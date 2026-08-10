//! Mapper for `control_msgs/msg/MotionArgument`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn motion_argument_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::MotionArgument> {
    Ok(crate::control_msgs::msg::v1::MotionArgument {
        name: read_string(view, "name")?,
        value: read_f64(view, "value")?,
    })
}

pub(crate) fn motion_argument_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::MotionArgument,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    write_f64(view, "value", bus.value)?;
    Ok(())
}

pub(crate) fn motion_argument_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::MotionArgument> {
    motion_argument_from_view(&msg.view())
}

pub(crate) fn motion_argument_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::MotionArgument,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/MotionArgument")?;
    motion_argument_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsMotionArgumentMapper;
impl TopicMapper for ControlMsgsMotionArgumentMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MotionArgument"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(motion_argument_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::MotionArgument as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode control_msgs/msg/MotionArgument: {e}"))
        })?;
        motion_argument_bus_to_dyn(&bus)
    }
}
