#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/multi_dof.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/control_msgs/single_dof_state.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/multi_dof_state_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::MultiDOFStateStamped multi_dof_state_stamped_to_bus(const ::control_msgs::msg::MultiDOFStateStamped &msg) {
  ::control_msgs::msg::v1::MultiDOFStateStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.dof_states) {
    *bus.add_dof_states() = ::robot_bus::ros2_bridge_mappers::control_msgs::single_dof_state_to_bus(x);
  }
  return bus;
}

inline ::control_msgs::msg::MultiDOFStateStamped multi_dof_state_stamped_to_ros(const ::control_msgs::msg::v1::MultiDOFStateStamped &bus) {
  ::control_msgs::msg::MultiDOFStateStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.dof_states.clear();
  for (const auto &x : bus.dof_states()) {
    out.dof_states.push_back(::robot_bus::ros2_bridge_mappers::control_msgs::single_dof_state_to_ros(x));
  }
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsMultiDofStateStampedMapper
    : public TypedTopicMapper<ControlMsgsMultiDofStateStampedMapper, ::control_msgs::msg::MultiDOFStateStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::MultiDOFStateStamped &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::multi_dof_state_stamped_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::MultiDOFStateStamped bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::MultiDOFStateStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::multi_dof_state_stamped_to_ros(bus);
  }
};
#else
struct ControlMsgsMultiDofStateStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
