//! Mapper for `trajectory_msgs/msg/JointTrajectoryPoint`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn joint_trajectory_point_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::trajectory_msgs::msg::v1::JointTrajectoryPoint> {
    Ok(crate::trajectory_msgs::msg::v1::JointTrajectoryPoint {
        positions: read_f64_seq(view, "positions")?,
        velocities: read_f64_seq(view, "velocities")?,
        accelerations: read_f64_seq(view, "accelerations")?,
        effort: read_f64_seq(view, "effort")?,
        time_from_start: nested_view(view, "time_from_start")?
            .as_ref()
            .map(super::super::builtin_interfaces::duration::duration_from_view)
            .transpose()?,
    })
}

pub(crate) fn joint_trajectory_point_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::trajectory_msgs::msg::v1::JointTrajectoryPoint,
) -> Result<()> {
    write_f64_seq(view, "positions", &bus.positions)?;
    write_f64_seq(view, "velocities", &bus.velocities)?;
    write_f64_seq(view, "accelerations", &bus.accelerations)?;
    write_f64_seq(view, "effort", &bus.effort)?;
    if let Some(v) = &bus.time_from_start {
        with_nested_mut(view, "time_from_start", |nested| {
            super::super::builtin_interfaces::duration::duration_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn joint_trajectory_point_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::trajectory_msgs::msg::v1::JointTrajectoryPoint> {
    joint_trajectory_point_from_view(&msg.view())
}

pub(crate) fn joint_trajectory_point_bus_to_dyn(
    bus: &crate::trajectory_msgs::msg::v1::JointTrajectoryPoint,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("trajectory_msgs/msg/JointTrajectoryPoint")?;
    joint_trajectory_point_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct TrajectoryMsgsJointTrajectoryPointMapper;
impl TopicMapper for TrajectoryMsgsJointTrajectoryPointMapper {
    fn type_name(&self) -> &'static str {
        "trajectory_msgs/msg/JointTrajectoryPoint"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_trajectory_point_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::trajectory_msgs::msg::v1::JointTrajectoryPoint as ProstMessage>::decode(
            payload,
        )
        .map_err(|e| {
            BusError::Protocol(format!(
                "decode trajectory_msgs/msg/JointTrajectoryPoint: {e}"
            ))
        })?;
        joint_trajectory_point_bus_to_dyn(&bus)
    }
}
