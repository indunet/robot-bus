#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/stereo_msgs/msg/v1/disparity_image.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/image.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/region_of_interest.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <stereo_msgs/msg/disparity_image.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace stereo_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::stereo_msgs::msg::v1::DisparityImage disparity_image_to_bus(const ::stereo_msgs::msg::DisparityImage &msg) {
  ::stereo_msgs::msg::v1::DisparityImage bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_image() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::image_to_bus(msg.image);
  bus.set_f(msg.f);
  bus.set_t(msg.t);
  *bus.mutable_valid_window() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::region_of_interest_to_bus(msg.valid_window);
  bus.set_min_disparity(msg.min_disparity);
  bus.set_max_disparity(msg.max_disparity);
  bus.set_delta_d(msg.delta_d);
  return bus;
}

inline ::stereo_msgs::msg::DisparityImage disparity_image_to_ros(const ::stereo_msgs::msg::v1::DisparityImage &bus) {
  ::stereo_msgs::msg::DisparityImage out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.image = ::robot_bus::ros2_bridge_mappers::sensor_msgs::image_to_ros(bus.image());
  out.f = bus.f();
  out.t = bus.t();
  out.valid_window = ::robot_bus::ros2_bridge_mappers::sensor_msgs::region_of_interest_to_ros(bus.valid_window());
  out.min_disparity = bus.min_disparity();
  out.max_disparity = bus.max_disparity();
  out.delta_d = bus.delta_d();
  return out;
}
#endif

}  // namespace stereo_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StereoMsgsDisparityImageMapper
    : public TypedTopicMapper<StereoMsgsDisparityImageMapper, ::stereo_msgs::msg::DisparityImage> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::stereo_msgs::msg::DisparityImage &msg) const {
    auto bus = ros2_bridge_mappers::stereo_msgs::disparity_image_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::stereo_msgs::msg::DisparityImage bus_to_ros(BytesView payload) const {
    ::stereo_msgs::msg::v1::DisparityImage bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::stereo_msgs::disparity_image_to_ros(bus);
  }
};
#else
struct StereoMsgsDisparityImageMapper : TopicMapper {};
#endif

}  // namespace robot_bus
