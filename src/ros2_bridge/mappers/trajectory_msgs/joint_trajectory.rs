//! Mapper for `trajectory_msgs/msg/JointTrajectory`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn joint_trajectory_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::trajectory_msgs::msg::v1::JointTrajectory> {
    Ok(crate::trajectory_msgs::msg::v1::JointTrajectory {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        joint_names: read_string_seq(view, "joint_names")?,
        points: read_message_seq(
            view,
            "points",
            super::joint_trajectory_point::joint_trajectory_point_from_view,
        )?,
    })
}

pub(crate) fn joint_trajectory_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::trajectory_msgs::msg::v1::JointTrajectory,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_string_seq(view, "joint_names", &bus.joint_names)?;
    write_message_seq(
        view,
        "points",
        &bus.points,
        super::joint_trajectory_point::joint_trajectory_point_write,
    )?;
    Ok(())
}

pub(crate) fn joint_trajectory_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::trajectory_msgs::msg::v1::JointTrajectory> {
    joint_trajectory_from_view(&msg.view())
}

pub(crate) fn joint_trajectory_bus_to_dyn(
    bus: &crate::trajectory_msgs::msg::v1::JointTrajectory,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("trajectory_msgs/msg/JointTrajectory")?;
    joint_trajectory_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct TrajectoryMsgsJointTrajectoryMapper;
impl TopicMapper for TrajectoryMsgsJointTrajectoryMapper {
    fn type_name(&self) -> &'static str {
        "trajectory_msgs/msg/JointTrajectory"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joint_trajectory_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::trajectory_msgs::msg::v1::JointTrajectory as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode trajectory_msgs/msg/JointTrajectory: {e}"))
                })?;
        joint_trajectory_bus_to_dyn(&bus)
    }
}
