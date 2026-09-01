#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/single_dof_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/control_msgs/single_dof_state.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/single_dof_state_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::SingleDOFStateStamped single_dof_state_stamped_to_bus(const ::control_msgs::msg::SingleDOFStateStamped &msg) {
  ::control_msgs::msg::v1::SingleDOFStateStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_state() = ::robot_bus::ros2_bridge_mappers::control_msgs::single_dof_state_to_bus(msg.state);
  return bus;
}

inline ::control_msgs::msg::SingleDOFStateStamped single_dof_state_stamped_to_ros(const ::control_msgs::msg::v1::SingleDOFStateStamped &bus) {
  ::control_msgs::msg::SingleDOFStateStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.state = ::robot_bus::ros2_bridge_mappers::control_msgs::single_dof_state_to_ros(bus.state());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsSingleDofStateStampedMapper
    : public TypedTopicMapper<ControlMsgsSingleDofStateStampedMapper, ::control_msgs::msg::SingleDOFStateStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::SingleDOFStateStamped &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::single_dof_state_stamped_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::SingleDOFStateStamped bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::SingleDOFStateStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::single_dof_state_stamped_to_ros(bus);
  }
};
#else
struct ControlMsgsSingleDofStateStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
