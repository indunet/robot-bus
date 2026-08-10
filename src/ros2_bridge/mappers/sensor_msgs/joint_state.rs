//! Mapper for `sensor_msgs/msg/JointState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joint_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::JointState> {
    Ok(crate::sensor_msgs::msg::v1::JointState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        name: read_string_seq(view, "name")?,
        position: read_f64_seq(view, "position")?,
        velocity: read_f64_seq(view, "velocity")?,
        effort: read_f64_seq(view, "effort")?,
    })
}

pub(crate) fn joint_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::JointState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string_seq(view, "name", &bus.name)?;
    write_f64_seq(view, "position", &bus.position)?;
    write_f64_seq(view, "velocity", &bus.velocity)?;
    write_f64_seq(view, "effort", &bus.effort)?;
    Ok(())
}

pub(crate) fn joint_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::JointState> {
    joint_state_from_view(&msg.view())
}

pub(crate) fn joint_state_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::JointState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/JointState")?;
    joint_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsJointStateMapper;
impl TopicMapper for SensorMsgsJointStateMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/JointState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::JointState as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/JointState: {e}")))?;
        joint_state_bus_to_dyn(&bus)
    }
}
