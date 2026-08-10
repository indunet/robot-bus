//! Mapper for `foxglove_msgs/msg/JointStates`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joint_states_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::JointStates> {
    Ok(crate::foxglove_msgs::msg::v1::JointStates {
        timestamp: read_timestamp(view, "timestamp")?,
        joints: read_message_seq(view, "joints", super::joint_state::joint_state_from_view)?,
    })
}

pub(crate) fn joint_states_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::JointStates,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_message_seq(view, "joints", &bus.joints, super::joint_state::joint_state_write)?;
    Ok(())
}

pub(crate) fn joint_states_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::JointStates> {
    joint_states_from_view(&msg.view())
}

pub(crate) fn joint_states_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::JointStates,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/JointStates")?;
    joint_states_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsJointStatesMapper;
impl TopicMapper for FoxgloveMsgsJointStatesMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/JointStates"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_states_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::JointStates as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/JointStates: {e}"))
            })?;
        joint_states_bus_to_dyn(&bus)
    }
}
