#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/dynamic_joint_state.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/interface_value.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::InterfaceValue interface_value_to_bus(const ::control_msgs::msg::InterfaceValue &msg) {
  ::control_msgs::msg::v1::InterfaceValue bus;
  for (const auto &x : msg.interface_names) {
    bus.add_interface_names(x.c_str());
  }
  for (auto x : msg.values) {
    bus.add_values(x);
  }
  return bus;
}

inline ::control_msgs::msg::InterfaceValue interface_value_to_ros(const ::control_msgs::msg::v1::InterfaceValue &bus) {
  ::control_msgs::msg::InterfaceValue out;
  out.interface_names.clear();
  for (const auto &x : bus.interface_names()) {
    out.interface_names.push_back(x);
  }
  out.values.assign(bus.values().begin(), bus.values().end());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsInterfaceValueMapper
    : public TypedTopicMapper<ControlMsgsInterfaceValueMapper, ::control_msgs::msg::InterfaceValue> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::InterfaceValue &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::interface_value_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::InterfaceValue bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::InterfaceValue bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::interface_value_to_ros(bus);
  }
};
#else
struct ControlMsgsInterfaceValueMapper : TopicMapper {};
#endif

}  // namespace robot_bus
