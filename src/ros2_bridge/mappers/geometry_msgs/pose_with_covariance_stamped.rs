//! Mapper for `geometry_msgs/msg/PoseWithCovarianceStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn pose_with_covariance_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped> {
    Ok(crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose_with_covariance::pose_with_covariance_from_view)
            .transpose()?,
    })
}

pub(crate) fn pose_with_covariance_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::pose_with_covariance::pose_with_covariance_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn pose_with_covariance_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped> {
    pose_with_covariance_stamped_from_view(&msg.view())
}

pub(crate) fn pose_with_covariance_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/PoseWithCovarianceStamped")?;
    pose_with_covariance_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPoseWithCovarianceStampedMapper;
impl TopicMapper for GeometryMsgsPoseWithCovarianceStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PoseWithCovarianceStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(pose_with_covariance_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::geometry_msgs::msg::v1::PoseWithCovarianceStamped as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode geometry_msgs/msg/PoseWithCovarianceStamped: {e}"
                ))
            })?;
        pose_with_covariance_stamped_bus_to_dyn(&bus)
    }
}
