//! Typed mapper for `stereo_msgs/msg/DisparityImage`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn disparity_image_to_bus(
    msg: ros_env::stereo_msgs::msg::DisparityImage,
) -> robot_bus::stereo_msgs::msg::v1::DisparityImage {
    robot_bus::stereo_msgs::msg::v1::DisparityImage {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        image: Some(crate::mappers::sensor_msgs::image::image_to_bus(msg.image)),
        f: msg.f,
        t: msg.t,
        valid_window: Some(
            crate::mappers::sensor_msgs::region_of_interest::region_of_interest_to_bus(
                msg.valid_window,
            ),
        ),
        min_disparity: msg.min_disparity,
        max_disparity: msg.max_disparity,
        delta_d: msg.delta_d,
    }
}

pub(crate) fn disparity_image_to_ros(
    bus: robot_bus::stereo_msgs::msg::v1::DisparityImage,
) -> ros_env::stereo_msgs::msg::DisparityImage {
    ros_env::stereo_msgs::msg::DisparityImage {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        image: crate::mappers::sensor_msgs::image::image_to_ros(
            bus.image.unwrap_or_default(),
        ),
        f: bus.f,
        t: bus.t,
        valid_window:
            crate::mappers::sensor_msgs::region_of_interest::region_of_interest_to_ros(
                bus.valid_window.unwrap_or_default(),
            ),
        min_disparity: bus.min_disparity,
        max_disparity: bus.max_disparity,
        delta_d: bus.delta_d,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StereoMsgsDisparityImageMapper;

impl TypedTopicMapper for StereoMsgsDisparityImageMapper {
    type Ros = ros_env::stereo_msgs::msg::DisparityImage;
    type Bus = robot_bus::stereo_msgs::msg::v1::DisparityImage;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(disparity_image_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(disparity_image_to_ros(msg))
    }
}
