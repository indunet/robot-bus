#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/diagnostic_msgs/msg/v1/diagnostic.pb.h>
#include <robot_bus/ros2_bridge/mappers/diagnostic_msgs/key_value.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <diagnostic_msgs/msg/diagnostic_status.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace diagnostic_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::diagnostic_msgs::msg::v1::DiagnosticStatus diagnostic_status_to_bus(const ::diagnostic_msgs::msg::DiagnosticStatus &msg) {
  ::diagnostic_msgs::msg::v1::DiagnosticStatus bus;
  bus.set_level(msg.level);
  bus.set_name(msg.name.c_str());
  bus.set_message(msg.message.c_str());
  bus.set_hardware_id(msg.hardware_id.c_str());
  for (const auto &x : msg.values) {
    *bus.add_values() = ::robot_bus::ros2_bridge_mappers::diagnostic_msgs::key_value_to_bus(x);
  }
  return bus;
}

inline ::diagnostic_msgs::msg::DiagnosticStatus diagnostic_status_to_ros(const ::diagnostic_msgs::msg::v1::DiagnosticStatus &bus) {
  ::diagnostic_msgs::msg::DiagnosticStatus out;
  out.level = bus.level();
  out.name = bus.name();
  out.message = bus.message();
  out.hardware_id = bus.hardware_id();
  out.values.clear();
  for (const auto &x : bus.values()) {
    out.values.push_back(::robot_bus::ros2_bridge_mappers::diagnostic_msgs::key_value_to_ros(x));
  }
  return out;
}
#endif

}  // namespace diagnostic_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class DiagnosticMsgsDiagnosticStatusMapper
    : public TypedTopicMapper<DiagnosticMsgsDiagnosticStatusMapper, ::diagnostic_msgs::msg::DiagnosticStatus> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::diagnostic_msgs::msg::DiagnosticStatus &msg) const {
    auto bus = ros2_bridge_mappers::diagnostic_msgs::diagnostic_status_to_bus(msg);
    return encode_pb(bus);
  }

  ::diagnostic_msgs::msg::DiagnosticStatus bus_to_ros(BytesView payload) const {
    ::diagnostic_msgs::msg::v1::DiagnosticStatus bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::diagnostic_msgs::diagnostic_status_to_ros(bus);
  }
};
#else
struct DiagnosticMsgsDiagnosticStatusMapper : TopicMapper {};
#endif

}  // namespace robot_bus
