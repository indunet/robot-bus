//! Mapper for `sensor_msgs/msg/MultiDOFJointState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn multi_dof_joint_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::MultiDofJointState> {
    Ok(crate::sensor_msgs::msg::v1::MultiDofJointState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        joint_names: read_string_seq(view, "joint_names")?,
        transforms: read_message_seq(
            view,
            "transforms",
            super::super::geometry_msgs::transform::transform_from_view,
        )?,
        twist: read_message_seq(
            view,
            "twist",
            super::super::geometry_msgs::twist::twist_from_view,
        )?,
        wrench: read_message_seq(
            view,
            "wrench",
            super::super::geometry_msgs::wrench::wrench_from_view,
        )?,
    })
}

pub(crate) fn multi_dof_joint_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::MultiDofJointState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string_seq(view, "joint_names", &bus.joint_names)?;
    write_message_seq(
        view,
        "transforms",
        &bus.transforms,
        super::super::geometry_msgs::transform::transform_write,
    )?;
    write_message_seq(
        view,
        "twist",
        &bus.twist,
        super::super::geometry_msgs::twist::twist_write,
    )?;
    write_message_seq(
        view,
        "wrench",
        &bus.wrench,
        super::super::geometry_msgs::wrench::wrench_write,
    )?;
    Ok(())
}

pub(crate) fn multi_dof_joint_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::MultiDofJointState> {
    multi_dof_joint_state_from_view(&msg.view())
}

pub(crate) fn multi_dof_joint_state_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::MultiDofJointState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/MultiDOFJointState")?;
    multi_dof_joint_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsMultiDofJointStateMapper;
impl TopicMapper for SensorMsgsMultiDofJointStateMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/MultiDOFJointState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_dof_joint_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::sensor_msgs::msg::v1::MultiDofJointState as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode sensor_msgs/msg/MultiDOFJointState: {e}"))
                })?;
        multi_dof_joint_state_bus_to_dyn(&bus)
    }
}
