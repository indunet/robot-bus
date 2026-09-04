#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/camera_info.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/region_of_interest.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/camera_info.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::CameraInfo camera_info_to_bus(const ::sensor_msgs::msg::CameraInfo &msg) {
  ::sensor_msgs::msg::v1::CameraInfo bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_height(msg.height);
  bus.set_width(msg.width);
  bus.set_distortion_model(msg.distortion_model.c_str());
  for (auto x : msg.d) {
    bus.add_d(x);
  }
  for (auto x : msg.k) {
    bus.add_k(x);
  }
  for (auto x : msg.r) {
    bus.add_r(x);
  }
  for (auto x : msg.p) {
    bus.add_p(x);
  }
  bus.set_binning_x(msg.binning_x);
  bus.set_binning_y(msg.binning_y);
  *bus.mutable_roi() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::region_of_interest_to_bus(msg.roi);
  return bus;
}

inline ::sensor_msgs::msg::CameraInfo camera_info_to_ros(const ::sensor_msgs::msg::v1::CameraInfo &bus) {
  ::sensor_msgs::msg::CameraInfo out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.height = bus.height();
  out.width = bus.width();
  out.distortion_model = bus.distortion_model();
  out.d.assign(bus.d().begin(), bus.d().end());
  out.k.assign(bus.k().begin(), bus.k().end());
  out.r.assign(bus.r().begin(), bus.r().end());
  out.p.assign(bus.p().begin(), bus.p().end());
  out.binning_x = bus.binning_x();
  out.binning_y = bus.binning_y();
  out.roi = ::robot_bus::ros2_bridge_mappers::sensor_msgs::region_of_interest_to_ros(bus.roi());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsCameraInfoMapper
    : public TypedTopicMapper<SensorMsgsCameraInfoMapper, ::sensor_msgs::msg::CameraInfo> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::CameraInfo &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::camera_info_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::CameraInfo bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::CameraInfo bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::camera_info_to_ros(bus);
  }
};
#else
struct SensorMsgsCameraInfoMapper : TopicMapper {};
#endif

}  // namespace robot_bus
