#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/single_dof_state.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/single_dof_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::SingleDOFState single_dof_state_to_bus(const ::control_msgs::msg::SingleDOFState &msg) {
  ::control_msgs::msg::v1::SingleDOFState bus;
  bus.set_name(msg.name.c_str());
  bus.set_reference(msg.reference);
  bus.set_feedback(msg.feedback);
  bus.set_feedback_dot(msg.feedback_dot);
  bus.set_error(msg.error);
  bus.set_error_dot(msg.error_dot);
  bus.set_time_step(msg.time_step);
  bus.set_output(msg.output);
  return bus;
}

inline ::control_msgs::msg::SingleDOFState single_dof_state_to_ros(const ::control_msgs::msg::v1::SingleDOFState &bus) {
  ::control_msgs::msg::SingleDOFState out;
  out.name = bus.name();
  out.reference = bus.reference();
  out.feedback = bus.feedback();
  out.feedback_dot = bus.feedback_dot();
  out.error = bus.error();
  out.error_dot = bus.error_dot();
  out.time_step = bus.time_step();
  out.output = bus.output();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsSingleDofStateMapper
    : public TypedTopicMapper<ControlMsgsSingleDofStateMapper, ::control_msgs::msg::SingleDOFState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::SingleDOFState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::single_dof_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::SingleDOFState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::SingleDOFState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::single_dof_state_to_ros(bus);
  }
};
#else
struct ControlMsgsSingleDofStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
