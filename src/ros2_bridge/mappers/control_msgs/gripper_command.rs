//! Mapper for `control_msgs/msg/GripperCommand`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn gripper_command_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::GripperCommand> {
    Ok(crate::control_msgs::msg::v1::GripperCommand {
        position: read_f64(view, "position")?,
        max_effort: read_f64(view, "max_effort")?,
    })
}

pub(crate) fn gripper_command_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::GripperCommand,
) -> Result<()> {
    write_f64(view, "position", bus.position)?;
    write_f64(view, "max_effort", bus.max_effort)?;
    Ok(())
}

pub(crate) fn gripper_command_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::GripperCommand> {
    gripper_command_from_view(&msg.view())
}

pub(crate) fn gripper_command_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::GripperCommand,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/GripperCommand")?;
    gripper_command_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsGripperCommandMapper;
impl TopicMapper for ControlMsgsGripperCommandMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/GripperCommand"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(gripper_command_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::GripperCommand as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode control_msgs/msg/GripperCommand: {e}"))
        })?;
        gripper_command_bus_to_dyn(&bus)
    }
}
