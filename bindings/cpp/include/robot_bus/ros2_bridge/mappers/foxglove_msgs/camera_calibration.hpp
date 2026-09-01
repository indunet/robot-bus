#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/camera_calibration.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/camera_calibration.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::CameraCalibration camera_calibration_to_bus(const ::foxglove_msgs::msg::CameraCalibration &msg) {
  ::foxglove_msgs::msg::v1::CameraCalibration bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  bus.set_width(msg.width);
  bus.set_height(msg.height);
  bus.set_distortion_model(msg.distortion_model.c_str());
  for (auto x : msg.D) {
    bus.add_D(x);
  }
  for (auto x : msg.K) {
    bus.add_K(x);
  }
  for (auto x : msg.R) {
    bus.add_R(x);
  }
  for (auto x : msg.P) {
    bus.add_P(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::CameraCalibration camera_calibration_to_ros(const ::foxglove_msgs::msg::v1::CameraCalibration &bus) {
  ::foxglove_msgs::msg::CameraCalibration out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.width = bus.width();
  out.height = bus.height();
  out.distortion_model = bus.distortion_model();
  out.D.assign(bus.D().begin(), bus.D().end());
  out.K.assign(bus.K().begin(), bus.K().end());
  out.R.assign(bus.R().begin(), bus.R().end());
  out.P.assign(bus.P().begin(), bus.P().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsCameraCalibrationMapper
    : public TypedTopicMapper<FoxgloveMsgsCameraCalibrationMapper, ::foxglove_msgs::msg::CameraCalibration> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::CameraCalibration &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::camera_calibration_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::CameraCalibration bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::CameraCalibration bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::camera_calibration_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsCameraCalibrationMapper : TopicMapper {};
#endif

}  // namespace robot_bus
