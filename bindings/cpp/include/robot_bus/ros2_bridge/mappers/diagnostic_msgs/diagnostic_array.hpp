#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/diagnostic_msgs/msg/v1/diagnostic.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/diagnostic_msgs/diagnostic_status.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <diagnostic_msgs/msg/diagnostic_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace diagnostic_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::diagnostic_msgs::msg::v1::DiagnosticArray diagnostic_array_to_bus(const ::diagnostic_msgs::msg::DiagnosticArray &msg) {
  ::diagnostic_msgs::msg::v1::DiagnosticArray bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.status) {
    *bus.add_status() = ::robot_bus::ros2_bridge_mappers::diagnostic_msgs::diagnostic_status_to_bus(x);
  }
  return bus;
}

inline ::diagnostic_msgs::msg::DiagnosticArray diagnostic_array_to_ros(const ::diagnostic_msgs::msg::v1::DiagnosticArray &bus) {
  ::diagnostic_msgs::msg::DiagnosticArray out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.status.clear();
  for (const auto &x : bus.status()) {
    out.status.push_back(::robot_bus::ros2_bridge_mappers::diagnostic_msgs::diagnostic_status_to_ros(x));
  }
  return out;
}
#endif

}  // namespace diagnostic_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class DiagnosticMsgsDiagnosticArrayMapper
    : public TypedTopicMapper<DiagnosticMsgsDiagnosticArrayMapper, ::diagnostic_msgs::msg::DiagnosticArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::diagnostic_msgs::msg::DiagnosticArray &msg) const {
    auto bus = ros2_bridge_mappers::diagnostic_msgs::diagnostic_array_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::diagnostic_msgs::msg::DiagnosticArray bus_to_ros(BytesView payload) const {
    ::diagnostic_msgs::msg::v1::DiagnosticArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::diagnostic_msgs::diagnostic_array_to_ros(bus);
  }
};
#else
struct DiagnosticMsgsDiagnosticArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
