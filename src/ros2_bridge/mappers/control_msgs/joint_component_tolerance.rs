//! Mapper for `control_msgs/msg/JointComponentTolerance`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joint_component_tolerance_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::JointComponentTolerance> {
    Ok(crate::control_msgs::msg::v1::JointComponentTolerance {
        joint_name: read_string(view, "joint_name")?,
        component: read_u32(view, "component")?,
        value: read_f64(view, "value")?,
    })
}

pub(crate) fn joint_component_tolerance_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::JointComponentTolerance,
) -> Result<()> {
    write_string(view, "joint_name", &bus.joint_name)?;
    write_u32(view, "component", bus.component)?;
    write_f64(view, "value", bus.value)?;
    Ok(())
}

pub(crate) fn joint_component_tolerance_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::JointComponentTolerance> {
    joint_component_tolerance_from_view(&msg.view())
}

pub(crate) fn joint_component_tolerance_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::JointComponentTolerance,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/JointComponentTolerance")?;
    joint_component_tolerance_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsJointComponentToleranceMapper;
impl TopicMapper for ControlMsgsJointComponentToleranceMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/JointComponentTolerance"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_component_tolerance_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::JointComponentTolerance as ProstMessage>::decode(
            payload,
        )
        .map_err(|e| {
            BusError::Protocol(format!(
                "decode control_msgs/msg/JointComponentTolerance: {e}"
            ))
        })?;
        joint_component_tolerance_bus_to_dyn(&bus)
    }
}
