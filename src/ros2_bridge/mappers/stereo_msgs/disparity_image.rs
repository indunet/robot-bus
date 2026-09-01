//! Typed mapper for `stereo_msgs/msg/DisparityImage`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn disparity_image_to_bus(msg: ros_env::stereo_msgs::msg::DisparityImage) -> crate::stereo_msgs::msg::v1::DisparityImage {
    crate::stereo_msgs::msg::v1::DisparityImage {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        image: Some(crate::ros2_bridge::mappers::sensor_msgs::image::image_to_bus(msg.image)),
        f: msg.f,
        t: msg.t,
        valid_window: Some(crate::ros2_bridge::mappers::sensor_msgs::region_of_interest::region_of_interest_to_bus(msg.valid_window)),
        min_disparity: msg.min_disparity,
        max_disparity: msg.max_disparity,
        delta_d: msg.delta_d,
    }
}

pub(crate) fn disparity_image_to_ros(bus: crate::stereo_msgs::msg::v1::DisparityImage) -> ros_env::stereo_msgs::msg::DisparityImage {
    ros_env::stereo_msgs::msg::DisparityImage {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        image: crate::ros2_bridge::mappers::sensor_msgs::image::image_to_ros(bus.image.unwrap_or_default()),
        f: bus.f,
        t: bus.t,
        valid_window: crate::ros2_bridge::mappers::sensor_msgs::region_of_interest::region_of_interest_to_ros(bus.valid_window.unwrap_or_default()),
        min_disparity: bus.min_disparity,
        max_disparity: bus.max_disparity,
        delta_d: bus.delta_d,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StereoMsgsDisparityImageMapper;

impl TypedTopicMapper for StereoMsgsDisparityImageMapper {
    type Ros = ros_env::stereo_msgs::msg::DisparityImage;
    type Bus = crate::stereo_msgs::msg::v1::DisparityImage;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(disparity_image_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(disparity_image_to_ros(msg))
    }
}
