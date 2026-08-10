//! Mapper for `control_msgs/msg/SteeringControllerStatus`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn steering_controller_status_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::SteeringControllerStatus> {
    Ok(crate::control_msgs::msg::v1::SteeringControllerStatus {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        traction_wheels_position: read_f64_seq(view, "traction_wheels_position")?,
        traction_wheels_velocity: read_f64_seq(view, "traction_wheels_velocity")?,
        steer_positions: read_f64_seq(view, "steer_positions")?,
        linear_velocity_command: read_f64_seq(view, "linear_velocity_command")?,
        steering_angle_command: read_f64_seq(view, "steering_angle_command")?,
    })
}

pub(crate) fn steering_controller_status_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::SteeringControllerStatus,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f64_seq(
        view,
        "traction_wheels_position",
        &bus.traction_wheels_position,
    )?;
    write_f64_seq(
        view,
        "traction_wheels_velocity",
        &bus.traction_wheels_velocity,
    )?;
    write_f64_seq(view, "steer_positions", &bus.steer_positions)?;
    write_f64_seq(
        view,
        "linear_velocity_command",
        &bus.linear_velocity_command,
    )?;
    write_f64_seq(view, "steering_angle_command", &bus.steering_angle_command)?;
    Ok(())
}

pub(crate) fn steering_controller_status_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::SteeringControllerStatus> {
    steering_controller_status_from_view(&msg.view())
}

pub(crate) fn steering_controller_status_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::SteeringControllerStatus,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/SteeringControllerStatus")?;
    steering_controller_status_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsSteeringControllerStatusMapper;
impl TopicMapper for ControlMsgsSteeringControllerStatusMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/SteeringControllerStatus"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(steering_controller_status_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::SteeringControllerStatus as ProstMessage>::decode(
            payload,
        )
        .map_err(|e| {
            BusError::Protocol(format!(
                "decode control_msgs/msg/SteeringControllerStatus: {e}"
            ))
        })?;
        steering_controller_status_bus_to_dyn(&bus)
    }
}
