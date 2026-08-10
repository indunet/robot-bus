//! Mapper for `control_msgs/msg/MultiDOFCommand`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn multi_dof_command_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::MultiDofCommand> {
    Ok(crate::control_msgs::msg::v1::MultiDofCommand {
        dof_names: read_string_seq(view, "dof_names")?,
        values: read_f64_seq(view, "values")?,
        values_dot: read_f64_seq(view, "values_dot")?,
    })
}

pub(crate) fn multi_dof_command_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::MultiDofCommand,
) -> Result<()> {
    write_string_seq(view, "dof_names", &bus.dof_names)?;
    write_f64_seq(view, "values", &bus.values)?;
    write_f64_seq(view, "values_dot", &bus.values_dot)?;
    Ok(())
}

pub(crate) fn multi_dof_command_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::MultiDofCommand> {
    multi_dof_command_from_view(&msg.view())
}

pub(crate) fn multi_dof_command_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::MultiDofCommand,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/MultiDOFCommand")?;
    multi_dof_command_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsMultiDofCommandMapper;
impl TopicMapper for ControlMsgsMultiDofCommandMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MultiDOFCommand"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_dof_command_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::MultiDofCommand as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode control_msgs/msg/MultiDOFCommand: {e}"))
            })?;
        multi_dof_command_bus_to_dyn(&bus)
    }
}
