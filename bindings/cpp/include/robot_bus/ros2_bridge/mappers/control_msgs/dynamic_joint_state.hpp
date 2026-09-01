#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/dynamic_joint_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/control_msgs/interface_value.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/dynamic_joint_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::DynamicJointState dynamic_joint_state_to_bus(const ::control_msgs::msg::DynamicJointState &msg) {
  ::control_msgs::msg::v1::DynamicJointState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  for (const auto &x : msg.interface_values) {
    *bus.add_interface_values() = ::robot_bus::ros2_bridge_mappers::control_msgs::interface_value_to_bus(x);
  }
  return bus;
}

inline ::control_msgs::msg::DynamicJointState dynamic_joint_state_to_ros(const ::control_msgs::msg::v1::DynamicJointState &bus) {
  ::control_msgs::msg::DynamicJointState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
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
class ControlMsgsDynamicJointStateMapper
    : public TypedTopicMapper<ControlMsgsDynamicJointStateMapper, ::control_msgs::msg::DynamicJointState> {
 public:
  const char *type_name() const override { return "control_msgs/msg/DynamicJointState"; }

  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::DynamicJointState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::dynamic_joint_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::DynamicJointState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::DynamicJointState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::dynamic_joint_state_to_ros(bus);
  }
};
#else
struct ControlMsgsDynamicJointStateMapper : TopicMapper {
  const char *type_name() const override { return "control_msgs/msg/DynamicJointState"; }
};
#endif

}  // namespace robot_bus
