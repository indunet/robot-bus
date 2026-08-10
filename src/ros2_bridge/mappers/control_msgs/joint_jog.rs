//! Mapper for `control_msgs/msg/JointJog`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joint_jog_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::JointJog> {
    Ok(crate::control_msgs::msg::v1::JointJog {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        joint_names: read_string_seq(view, "joint_names")?,
        displacements: read_f64_seq(view, "displacements")?,
        velocities: read_f64_seq(view, "velocities")?,
        duration: read_f64(view, "duration")?,
    })
}

pub(crate) fn joint_jog_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::JointJog,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string_seq(view, "joint_names", &bus.joint_names)?;
    write_f64_seq(view, "displacements", &bus.displacements)?;
    write_f64_seq(view, "velocities", &bus.velocities)?;
    write_f64(view, "duration", bus.duration)?;
    Ok(())
}

pub(crate) fn joint_jog_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::JointJog> {
    joint_jog_from_view(&msg.view())
}

pub(crate) fn joint_jog_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::JointJog,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/JointJog")?;
    joint_jog_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsJointJogMapper;
impl TopicMapper for ControlMsgsJointJogMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/JointJog"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_jog_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::JointJog as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode control_msgs/msg/JointJog: {e}")))?;
        joint_jog_bus_to_dyn(&bus)
    }
}
