//! Typed mapper for `sensor_msgs/msg/CameraInfo`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn camera_info_to_bus(msg: ros_env::sensor_msgs::msg::CameraInfo) -> crate::sensor_msgs::msg::v1::CameraInfo {
    crate::sensor_msgs::msg::v1::CameraInfo {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        height: msg.height.into(),
        width: msg.width.into(),
        distortion_model: crate::ros2_bridge::mappers::convert::from_ros_string(msg.distortion_model),
        d: crate::ros2_bridge::mappers::convert::f64_seq(msg.d),
        k: crate::ros2_bridge::mappers::convert::f64_seq(msg.k),
        r: crate::ros2_bridge::mappers::convert::f64_seq(msg.r),
        p: crate::ros2_bridge::mappers::convert::f64_seq(msg.p),
        binning_x: msg.binning_x.into(),
        binning_y: msg.binning_y.into(),
        roi: Some(crate::ros2_bridge::mappers::sensor_msgs::region_of_interest::region_of_interest_to_bus(msg.roi)),
    }
}

pub(crate) fn camera_info_to_ros(bus: crate::sensor_msgs::msg::v1::CameraInfo) -> ros_env::sensor_msgs::msg::CameraInfo {
    ros_env::sensor_msgs::msg::CameraInfo {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        height: bus.height as _,
        width: bus.width as _,
        distortion_model: crate::ros2_bridge::mappers::convert::to_ros_string(bus.distortion_model),
        d: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.d),
        k: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.k),
        r: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.r),
        p: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.p),
        binning_x: bus.binning_x as _,
        binning_y: bus.binning_y as _,
        roi: crate::ros2_bridge::mappers::sensor_msgs::region_of_interest::region_of_interest_to_ros(bus.roi.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsCameraInfoMapper;

impl TypedTopicMapper for SensorMsgsCameraInfoMapper {
    type Ros = ros_env::sensor_msgs::msg::CameraInfo;
    type Bus = crate::sensor_msgs::msg::v1::CameraInfo;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(camera_info_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(camera_info_to_ros(msg))
    }
}
