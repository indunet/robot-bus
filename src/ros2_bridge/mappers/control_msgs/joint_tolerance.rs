//! Mapper for `control_msgs/msg/JointTolerance`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn joint_tolerance_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::JointTolerance> {
    Ok(crate::control_msgs::msg::v1::JointTolerance {
        name: read_string(view, "name")?,
        position: read_f64(view, "position")?,
        velocity: read_f64(view, "velocity")?,
        acceleration: read_f64(view, "acceleration")?,
    })
}

pub(crate) fn joint_tolerance_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::JointTolerance,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    write_f64(view, "position", bus.position)?;
    write_f64(view, "velocity", bus.velocity)?;
    write_f64(view, "acceleration", bus.acceleration)?;
    Ok(())
}

pub(crate) fn joint_tolerance_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::JointTolerance> {
    joint_tolerance_from_view(&msg.view())
}

pub(crate) fn joint_tolerance_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::JointTolerance,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/JointTolerance")?;
    joint_tolerance_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsJointToleranceMapper;
impl TopicMapper for ControlMsgsJointToleranceMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/JointTolerance"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_tolerance_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::JointTolerance as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode control_msgs/msg/JointTolerance: {e}"))
        })?;
        joint_tolerance_bus_to_dyn(&bus)
    }
}
