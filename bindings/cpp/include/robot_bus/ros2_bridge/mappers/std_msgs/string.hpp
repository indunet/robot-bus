#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/string.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::String string_to_bus(const ::std_msgs::msg::String &msg) {
  ::std_msgs::msg::v1::String bus;
  bus.set_data(msg.data.c_str());
  return bus;
}

inline ::std_msgs::msg::String string_to_ros(const ::std_msgs::msg::v1::String &bus) {
  ::std_msgs::msg::String out;
  out.data = bus.data();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers
}  // namespace robot_bus
