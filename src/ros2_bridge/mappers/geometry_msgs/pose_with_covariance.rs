//! Mapper for `geometry_msgs/msg/PoseWithCovariance`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn pose_with_covariance_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::PoseWithCovariance> {
    Ok(crate::geometry_msgs::msg::v1::PoseWithCovariance {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        covariance: read_f64_seq(view, "covariance")?,
    })
}

pub(crate) fn pose_with_covariance_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::PoseWithCovariance,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_f64_seq(view, "covariance", &bus.covariance)?;
    Ok(())
}

pub(crate) fn pose_with_covariance_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::PoseWithCovariance> {
    pose_with_covariance_from_view(&msg.view())
}

pub(crate) fn pose_with_covariance_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::PoseWithCovariance,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/PoseWithCovariance")?;
    pose_with_covariance_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPoseWithCovarianceMapper;
impl TopicMapper for GeometryMsgsPoseWithCovarianceMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PoseWithCovariance"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(pose_with_covariance_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::geometry_msgs::msg::v1::PoseWithCovariance as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!("decode geometry_msgs/msg/PoseWithCovariance: {e}"))
                })?;
        pose_with_covariance_bus_to_dyn(&bus)
    }
}
