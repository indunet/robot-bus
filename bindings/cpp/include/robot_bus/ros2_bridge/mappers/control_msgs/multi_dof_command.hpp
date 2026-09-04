#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/multi_dof.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/multi_dof_command.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::MultiDOFCommand multi_dof_command_to_bus(const ::control_msgs::msg::MultiDOFCommand &msg) {
  ::control_msgs::msg::v1::MultiDOFCommand bus;
  for (const auto &x : msg.dof_names) {
    bus.add_dof_names(x.c_str());
  }
  for (auto x : msg.values) {
    bus.add_values(x);
  }
  for (auto x : msg.values_dot) {
    bus.add_values_dot(x);
  }
  return bus;
}

inline ::control_msgs::msg::MultiDOFCommand multi_dof_command_to_ros(const ::control_msgs::msg::v1::MultiDOFCommand &bus) {
  ::control_msgs::msg::MultiDOFCommand out;
  out.dof_names.clear();
  for (const auto &x : bus.dof_names()) {
    out.dof_names.push_back(x);
  }
  out.values.assign(bus.values().begin(), bus.values().end());
  out.values_dot.assign(bus.values_dot().begin(), bus.values_dot().end());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsMultiDofCommandMapper
    : public TypedTopicMapper<ControlMsgsMultiDofCommandMapper, ::control_msgs::msg::MultiDOFCommand> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::MultiDOFCommand &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::multi_dof_command_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::MultiDOFCommand bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::MultiDOFCommand bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::multi_dof_command_to_ros(bus);
  }
};
#else
struct ControlMsgsMultiDofCommandMapper : TopicMapper {};
#endif

}  // namespace robot_bus
