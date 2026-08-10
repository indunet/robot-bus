//! Mapper for `stereo_msgs/msg/DisparityImage`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn disparity_image_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::stereo_msgs::msg::v1::DisparityImage> {
    Ok(crate::stereo_msgs::msg::v1::DisparityImage {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        image: nested_view(view, "image")?
            .as_ref()
            .map(super::super::sensor_msgs::image::image_from_view)
            .transpose()?,
        f: read_f32(view, "f")?,
        t: read_f32(view, "t")?,
        valid_window: nested_view(view, "valid_window")?
            .as_ref()
            .map(super::super::sensor_msgs::region_of_interest::region_of_interest_from_view)
            .transpose()?,
        min_disparity: read_f32(view, "min_disparity")?,
        max_disparity: read_f32(view, "max_disparity")?,
        delta_d: read_f32(view, "delta_d")?,
    })
}

pub(crate) fn disparity_image_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::stereo_msgs::msg::v1::DisparityImage,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.image {
        with_nested_mut(view, "image", |nested| {
            super::super::sensor_msgs::image::image_write(nested, v)
        })?;
    }
    write_f32(view, "f", bus.f)?;
    write_f32(view, "t", bus.t)?;
    if let Some(v) = &bus.valid_window {
        with_nested_mut(view, "valid_window", |nested| {
            super::super::sensor_msgs::region_of_interest::region_of_interest_write(nested, v)
        })?;
    }
    write_f32(view, "min_disparity", bus.min_disparity)?;
    write_f32(view, "max_disparity", bus.max_disparity)?;
    write_f32(view, "delta_d", bus.delta_d)?;
    Ok(())
}

pub(crate) fn disparity_image_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::stereo_msgs::msg::v1::DisparityImage> {
    disparity_image_from_view(&msg.view())
}

pub(crate) fn disparity_image_bus_to_dyn(
    bus: &crate::stereo_msgs::msg::v1::DisparityImage,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("stereo_msgs/msg/DisparityImage")?;
    disparity_image_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct StereoMsgsDisparityImageMapper;
impl TopicMapper for StereoMsgsDisparityImageMapper {
    fn type_name(&self) -> &'static str {
        "stereo_msgs/msg/DisparityImage"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(disparity_image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::stereo_msgs::msg::v1::DisparityImage as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode stereo_msgs/msg/DisparityImage: {e}"))
            })?;
        disparity_image_bus_to_dyn(&bus)
    }
}
