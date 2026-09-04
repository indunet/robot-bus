#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/pid_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/duration.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/pid_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::PidState pid_state_to_bus(const ::control_msgs::msg::PidState &msg) {
  ::control_msgs::msg::v1::PidState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_timestep() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_bus(msg.timestep);
  bus.set_error(msg.error);
  bus.set_error_dot(msg.error_dot);
  bus.set_p_error(msg.p_error);
  bus.set_i_error(msg.i_error);
  bus.set_d_error(msg.d_error);
  bus.set_p_term(msg.p_term);
  bus.set_i_term(msg.i_term);
  bus.set_d_term(msg.d_term);
  bus.set_i_max(msg.i_max);
  bus.set_i_min(msg.i_min);
  bus.set_output(msg.output);
  return bus;
}

inline ::control_msgs::msg::PidState pid_state_to_ros(const ::control_msgs::msg::v1::PidState &bus) {
  ::control_msgs::msg::PidState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.timestep = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_ros(bus.timestep());
  out.error = bus.error();
  out.error_dot = bus.error_dot();
  out.p_error = bus.p_error();
  out.i_error = bus.i_error();
  out.d_error = bus.d_error();
  out.p_term = bus.p_term();
  out.i_term = bus.i_term();
  out.d_term = bus.d_term();
  out.i_max = bus.i_max();
  out.i_min = bus.i_min();
  out.output = bus.output();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsPidStateMapper
    : public TypedTopicMapper<ControlMsgsPidStateMapper, ::control_msgs::msg::PidState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::PidState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::pid_state_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::PidState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::PidState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::pid_state_to_ros(bus);
  }
};
#else
struct ControlMsgsPidStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
