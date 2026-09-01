#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/image.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/image.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::Image image_to_bus(const ::sensor_msgs::msg::Image &msg) {
  ::sensor_msgs::msg::v1::Image bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_height(msg.height);
  bus.set_width(msg.width);
  bus.set_encoding(msg.encoding.c_str());
  bus.set_is_bigendian(msg.is_bigendian != 0);
  bus.set_step(msg.step);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::sensor_msgs::msg::Image image_to_ros(const ::sensor_msgs::msg::v1::Image &bus) {
  ::sensor_msgs::msg::Image out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.height = bus.height();
  out.width = bus.width();
  out.encoding = bus.encoding();
  out.is_bigendian = bus.is_bigendian() ? 1 : 0;
  out.step = bus.step();
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers
}  // namespace robot_bus
