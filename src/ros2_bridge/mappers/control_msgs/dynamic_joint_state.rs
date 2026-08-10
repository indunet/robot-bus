//! Mapper for `control_msgs/msg/DynamicJointState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn dynamic_joint_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::DynamicJointState> {
    Ok(crate::control_msgs::msg::v1::DynamicJointState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        joint_names: read_string_seq(view, "joint_names")?,
        interface_values: read_message_seq(view, "interface_values", super::interface_value::interface_value_from_view)?,
    })
}

pub(crate) fn dynamic_joint_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::DynamicJointState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string_seq(view, "joint_names", &bus.joint_names)?;
    write_message_seq(
        view,
        "interface_values",
        &bus.interface_values,
        super::interface_value::interface_value_write,
    )?;
    Ok(())
}

pub(crate) fn dynamic_joint_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::DynamicJointState> {
    dynamic_joint_state_from_view(&msg.view())
}

pub(crate) fn dynamic_joint_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::DynamicJointState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/DynamicJointState")?;
    dynamic_joint_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsDynamicJointStateMapper;
impl TopicMapper for ControlMsgsDynamicJointStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/DynamicJointState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(dynamic_joint_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::DynamicJointState as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode control_msgs/msg/DynamicJointState: {e}"))
                })?;
        dynamic_joint_state_bus_to_dyn(&bus)
    }
}
