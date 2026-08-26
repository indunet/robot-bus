//! Mapper for `nav_msgs/msg/Path`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn path_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::Path> {
    Ok(crate::nav_msgs::msg::v1::Path {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        poses: read_message_seq(
            view,
            "poses",
            super::super::geometry_msgs::pose_stamped::pose_stamped_from_view,
        )?,
    })
}

pub(crate) fn path_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::Path,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "poses",
        &bus.poses,
        super::super::geometry_msgs::pose_stamped::pose_stamped_write,
    )?;
    Ok(())
}

pub(crate) fn path_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::Path> {
    path_from_view(&msg.view())
}

pub(crate) fn path_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::Path,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/Path")?;
    path_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsPathMapper;
impl TopicMapper for NavMsgsPathMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/Path"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(path_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::Path as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/Path: {e}")))?;
        path_bus_to_dyn(&bus)
    }
}
