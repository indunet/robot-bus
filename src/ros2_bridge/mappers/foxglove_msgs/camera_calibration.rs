//! Typed mapper for `foxglove_msgs/msg/CameraCalibration`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn camera_calibration_to_bus(msg: ros_env::foxglove_msgs::msg::CameraCalibration) -> crate::foxglove_msgs::msg::v1::CameraCalibration {
    crate::foxglove_msgs::msg::v1::CameraCalibration {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        width: msg.width,
        height: msg.height,
        distortion_model: crate::ros2_bridge::mappers::convert::from_ros_string(msg.distortion_model),
        d: crate::ros2_bridge::mappers::convert::f64_seq(msg.d),
        k: crate::ros2_bridge::mappers::convert::f64_seq(msg.k),
        r: crate::ros2_bridge::mappers::convert::f64_seq(msg.r),
        p: crate::ros2_bridge::mappers::convert::f64_seq(msg.p),
    }
}

pub(crate) fn camera_calibration_to_ros(bus: crate::foxglove_msgs::msg::v1::CameraCalibration) -> ros_env::foxglove_msgs::msg::CameraCalibration {
    ros_env::foxglove_msgs::msg::CameraCalibration {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        width: bus.width,
        height: bus.height,
        distortion_model: crate::ros2_bridge::mappers::convert::to_ros_string(bus.distortion_model),
        d: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.d),
        k: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.k),
        r: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.r),
        p: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.p),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCameraCalibrationMapper;

impl TypedTopicMapper for FoxgloveMsgsCameraCalibrationMapper {
    type Ros = ros_env::foxglove_msgs::msg::CameraCalibration;
    type Bus = crate::foxglove_msgs::msg::v1::CameraCalibration;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(camera_calibration_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(camera_calibration_to_ros(msg))
    }
}
