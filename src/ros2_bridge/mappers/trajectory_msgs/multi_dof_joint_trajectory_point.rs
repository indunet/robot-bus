//! Mapper for `trajectory_msgs/msg/MultiDOFJointTrajectoryPoint`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn multi_dof_joint_trajectory_point_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint> {
    Ok(
        crate::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint {
            transforms: read_message_seq(
                view,
                "transforms",
                super::super::geometry_msgs::transform::transform_from_view,
            )?,
            velocities: read_message_seq(
                view,
                "velocities",
                super::super::geometry_msgs::twist::twist_from_view,
            )?,
            accelerations: read_message_seq(
                view,
                "accelerations",
                super::super::geometry_msgs::twist::twist_from_view,
            )?,
            time_from_start: nested_view(view, "time_from_start")?
                .as_ref()
                .map(super::super::builtin_interfaces::duration::duration_from_view)
                .transpose()?,
        },
    )
}

pub(crate) fn multi_dof_joint_trajectory_point_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint,
) -> Result<()> {
    write_message_seq(
        view,
        "transforms",
        &bus.transforms,
        super::super::geometry_msgs::transform::transform_write,
    )?;
    write_message_seq(
        view,
        "velocities",
        &bus.velocities,
        super::super::geometry_msgs::twist::twist_write,
    )?;
    write_message_seq(
        view,
        "accelerations",
        &bus.accelerations,
        super::super::geometry_msgs::twist::twist_write,
    )?;
    if let Some(v) = &bus.time_from_start {
        with_nested_mut(view, "time_from_start", |nested| {
            super::super::builtin_interfaces::duration::duration_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn multi_dof_joint_trajectory_point_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint> {
    multi_dof_joint_trajectory_point_from_view(&msg.view())
}

pub(crate) fn multi_dof_joint_trajectory_point_bus_to_dyn(
    bus: &crate::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("trajectory_msgs/msg/MultiDOFJointTrajectoryPoint")?;
    multi_dof_joint_trajectory_point_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct TrajectoryMsgsMultiDofJointTrajectoryPointMapper;
impl TopicMapper for TrajectoryMsgsMultiDofJointTrajectoryPointMapper {
    fn type_name(&self) -> &'static str {
        "trajectory_msgs/msg/MultiDOFJointTrajectoryPoint"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_dof_joint_trajectory_point_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::trajectory_msgs::msg::v1::MultiDofJointTrajectoryPoint as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode trajectory_msgs/msg/MultiDOFJointTrajectoryPoint: {e}"))
            })?;
        multi_dof_joint_trajectory_point_bus_to_dyn(&bus)
    }
}
