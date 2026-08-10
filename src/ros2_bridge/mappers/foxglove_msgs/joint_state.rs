//! Mapper for `foxglove_msgs/msg/JointState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joint_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::JointState> {
    Ok(crate::foxglove_msgs::msg::v1::JointState {
        name: read_string(view, "name")?,
        position: read_f64_opt(view, "position")?,
        velocity: read_f64_opt(view, "velocity")?,
        acceleration: read_f64_opt(view, "acceleration")?,
        effort: read_f64_opt(view, "effort")?,
    })
}

pub(crate) fn joint_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::JointState,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    if let Some(v) = bus.position {
        write_f64(view, "position", v)?;
    }
    if let Some(v) = bus.velocity {
        write_f64(view, "velocity", v)?;
    }
    if let Some(v) = bus.acceleration {
        write_f64(view, "acceleration", v)?;
    }
    if let Some(v) = bus.effort {
        write_f64(view, "effort", v)?;
    }
    Ok(())
}

pub(crate) fn joint_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::JointState> {
    joint_state_from_view(&msg.view())
}

pub(crate) fn joint_state_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::JointState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/JointState")?;
    joint_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsJointStateMapper;
impl TopicMapper for FoxgloveMsgsJointStateMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/JointState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::JointState as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/JointState: {e}")))?;
        joint_state_bus_to_dyn(&bus)
    }
}
