//! Mapper for `control_msgs/msg/AdmittanceControllerState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn admittance_controller_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::AdmittanceControllerState> {
    Ok(crate::control_msgs::msg::v1::AdmittanceControllerState {
        ref_trans_base_fts: nested_view(view, "ref_trans_base_fts")?
            .as_ref()
            .map(super::super::geometry_msgs::transform_stamped::transform_stamped_from_view)
            .transpose()?,
        selected_axes: nested_view(view, "selected_axes")?
            .as_ref()
            .map(super::super::std_msgs::float64_multi_array::float64_multi_array_from_view)
            .transpose()?,
        ft_sensor_frame: nested_view(view, "ft_sensor_frame")?
            .as_ref()
            .map(super::super::geometry_msgs::transform_stamped::transform_stamped_from_view)
            .transpose()?,
        admittance_position: nested_view(view, "admittance_position")?
            .as_ref()
            .map(super::super::geometry_msgs::transform_stamped::transform_stamped_from_view)
            .transpose()?,
        admittance_acceleration: nested_view(view, "admittance_acceleration")?
            .as_ref()
            .map(super::super::geometry_msgs::twist_stamped::twist_stamped_from_view)
            .transpose()?,
        admittance_velocity: nested_view(view, "admittance_velocity")?
            .as_ref()
            .map(super::super::geometry_msgs::twist_stamped::twist_stamped_from_view)
            .transpose()?,
        wrench_base: nested_view(view, "wrench_base")?
            .as_ref()
            .map(super::super::geometry_msgs::wrench_stamped::wrench_stamped_from_view)
            .transpose()?,
        robot_ref_trans_base_fts: nested_view(view, "robot_ref_trans_base_fts")?
            .as_ref()
            .map(super::super::geometry_msgs::transform_stamped::transform_stamped_from_view)
            .transpose()?,
        joint_names: read_string_seq(view, "joint_names")?,
        joint_state: nested_view(view, "joint_state")?
            .as_ref()
            .map(super::super::sensor_msgs::joint_state::joint_state_from_view)
            .transpose()?,
    })
}

pub(crate) fn admittance_controller_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::AdmittanceControllerState,
) -> Result<()> {
    if let Some(v) = &bus.ref_trans_base_fts {
        with_nested_mut(view, "ref_trans_base_fts", |nested| {
            super::super::geometry_msgs::transform_stamped::transform_stamped_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.selected_axes {
        with_nested_mut(view, "selected_axes", |nested| {
            super::super::std_msgs::float64_multi_array::float64_multi_array_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.ft_sensor_frame {
        with_nested_mut(view, "ft_sensor_frame", |nested| {
            super::super::geometry_msgs::transform_stamped::transform_stamped_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.admittance_position {
        with_nested_mut(view, "admittance_position", |nested| {
            super::super::geometry_msgs::transform_stamped::transform_stamped_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.admittance_acceleration {
        with_nested_mut(view, "admittance_acceleration", |nested| {
            super::super::geometry_msgs::twist_stamped::twist_stamped_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.admittance_velocity {
        with_nested_mut(view, "admittance_velocity", |nested| {
            super::super::geometry_msgs::twist_stamped::twist_stamped_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.wrench_base {
        with_nested_mut(view, "wrench_base", |nested| {
            super::super::geometry_msgs::wrench_stamped::wrench_stamped_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.robot_ref_trans_base_fts {
        with_nested_mut(view, "robot_ref_trans_base_fts", |nested| {
            super::super::geometry_msgs::transform_stamped::transform_stamped_write(nested, v)
        })?;
    }
    write_string_seq(view, "joint_names", &bus.joint_names)?;
    if let Some(v) = &bus.joint_state {
        with_nested_mut(view, "joint_state", |nested| {
            super::super::sensor_msgs::joint_state::joint_state_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn admittance_controller_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::AdmittanceControllerState> {
    admittance_controller_state_from_view(&msg.view())
}

pub(crate) fn admittance_controller_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::AdmittanceControllerState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/AdmittanceControllerState")?;
    admittance_controller_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsAdmittanceControllerStateMapper;
impl TopicMapper for ControlMsgsAdmittanceControllerStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/AdmittanceControllerState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(admittance_controller_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::AdmittanceControllerState as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode control_msgs/msg/AdmittanceControllerState: {e}"
                ))
            })?;
        admittance_controller_state_bus_to_dyn(&bus)
    }
}
