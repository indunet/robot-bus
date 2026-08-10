//! Mapper for `control_msgs/msg/MecanumDriveControllerState`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn mecanum_drive_controller_state_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::MecanumDriveControllerState> {
    Ok(crate::control_msgs::msg::v1::MecanumDriveControllerState {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        front_left_wheel_velocity: read_f64(view, "front_left_wheel_velocity")?,
        front_right_wheel_velocity: read_f64(view, "front_right_wheel_velocity")?,
        back_left_wheel_velocity: read_f64(view, "back_left_wheel_velocity")?,
        back_right_wheel_velocity: read_f64(view, "back_right_wheel_velocity")?,
        reference_velocity: nested_view(view, "reference_velocity")?
            .as_ref()
            .map(super::super::geometry_msgs::twist::twist_from_view)
            .transpose()?,
    })
}

pub(crate) fn mecanum_drive_controller_state_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::MecanumDriveControllerState,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64(
        view,
        "front_left_wheel_velocity",
        bus.front_left_wheel_velocity,
    )?;
    write_f64(
        view,
        "front_right_wheel_velocity",
        bus.front_right_wheel_velocity,
    )?;
    write_f64(
        view,
        "back_left_wheel_velocity",
        bus.back_left_wheel_velocity,
    )?;
    write_f64(
        view,
        "back_right_wheel_velocity",
        bus.back_right_wheel_velocity,
    )?;
    if let Some(v) = &bus.reference_velocity {
        with_nested_mut(view, "reference_velocity", |nested| {
            super::super::geometry_msgs::twist::twist_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn mecanum_drive_controller_state_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::MecanumDriveControllerState> {
    mecanum_drive_controller_state_from_view(&msg.view())
}

pub(crate) fn mecanum_drive_controller_state_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::MecanumDriveControllerState,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/MecanumDriveControllerState")?;
    mecanum_drive_controller_state_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsMecanumDriveControllerStateMapper;
impl TopicMapper for ControlMsgsMecanumDriveControllerStateMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MecanumDriveControllerState"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(mecanum_drive_controller_state_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::MecanumDriveControllerState as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode control_msgs/msg/MecanumDriveControllerState: {e}"
                ))
            })?;
        mecanum_drive_controller_state_bus_to_dyn(&bus)
    }
}
