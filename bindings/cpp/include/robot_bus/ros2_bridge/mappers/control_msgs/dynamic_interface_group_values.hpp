#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/dynamic_joint_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/control_msgs/interface_value.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/dynamic_interface_group_values.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::DynamicInterfaceGroupValues dynamic_interface_group_values_to_bus(const ::control_msgs::msg::DynamicInterfaceGroupValues &msg) {
  ::control_msgs::msg::v1::DynamicInterfaceGroupValues bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.interface_groups) {
    bus.add_interface_groups(x.c_str());
  }
  for (const auto &x : msg.interface_values) {
    *bus.add_interface_values() = ::robot_bus::ros2_bridge_mappers::control_msgs::interface_value_to_bus(x);
  }
  return bus;
}

inline ::control_msgs::msg::DynamicInterfaceGroupValues dynamic_interface_group_values_to_ros(const ::control_msgs::msg::v1::DynamicInterfaceGroupValues &bus) {
  ::control_msgs::msg::DynamicInterfaceGroupValues out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.interface_groups.clear();
  for (const auto &x : bus.interface_groups()) {
    out.interface_groups.push_back(x);
  }
  out.interface_values.clear();
  for (const auto &x : bus.interface_values()) {
    out.interface_values.push_back(::robot_bus::ros2_bridge_mappers::control_msgs::interface_value_to_ros(x));
  }
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsDynamicInterfaceGroupValuesMapper
    : public TypedTopicMapper<ControlMsgsDynamicInterfaceGroupValuesMapper, ::control_msgs::msg::DynamicInterfaceGroupValues> {
 public:
  const char *type_name() const override { return "control_msgs/msg/DynamicInterfaceGroupValues"; }

  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::DynamicInterfaceGroupValues &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::dynamic_interface_group_values_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::DynamicInterfaceGroupValues bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::DynamicInterfaceGroupValues bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::dynamic_interface_group_values_to_ros(bus);
  }
};
#else
struct ControlMsgsDynamicInterfaceGroupValuesMapper : TopicMapper {
  const char *type_name() const override { return "control_msgs/msg/DynamicInterfaceGroupValues"; }
};
#endif

}  // namespace robot_bus
