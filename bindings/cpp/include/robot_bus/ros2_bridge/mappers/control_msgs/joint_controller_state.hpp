#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/joint_controller_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/joint_controller_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::JointControllerState joint_controller_state_to_bus(const ::control_msgs::msg::JointControllerState &msg) {
  ::control_msgs::msg::v1::JointControllerState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_set_point(msg.set_point);
  bus.set_process_value(msg.process_value);
  bus.set_process_value_dot(msg.process_value_dot);
  bus.set_error(msg.error);
  bus.set_time_step(msg.time_step);
  bus.set_command(msg.command);
  bus.set_p(msg.p);
  bus.set_i(msg.i);
  bus.set_d(msg.d);
  bus.set_i_clamp(msg.i_clamp);
  bus.set_antiwindup(msg.antiwindup);
  return bus;
}

inline ::control_msgs::msg::JointControllerState joint_controller_state_to_ros(const ::control_msgs::msg::v1::JointControllerState &bus) {
  ::control_msgs::msg::JointControllerState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.set_point = bus.set_point();
  out.process_value = bus.process_value();
  out.process_value_dot = bus.process_value_dot();
  out.error = bus.error();
  out.time_step = bus.time_step();
  out.command = bus.command();
  out.p = bus.p();
  out.i = bus.i();
  out.d = bus.d();
  out.i_clamp = bus.i_clamp();
  out.antiwindup = bus.antiwindup();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsJointControllerStateMapper
    : public TypedTopicMapper<ControlMsgsJointControllerStateMapper, ::control_msgs::msg::JointControllerState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::JointControllerState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::joint_controller_state_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::JointControllerState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::JointControllerState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::joint_controller_state_to_ros(bus);
  }
};
#else
struct ControlMsgsJointControllerStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
