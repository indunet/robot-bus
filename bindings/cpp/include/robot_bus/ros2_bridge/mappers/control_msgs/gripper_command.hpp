#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/gripper_command.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/gripper_command.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::GripperCommand gripper_command_to_bus(const ::control_msgs::msg::GripperCommand &msg) {
  ::control_msgs::msg::v1::GripperCommand bus;
  bus.set_position(msg.position);
  bus.set_max_effort(msg.max_effort);
  return bus;
}

inline ::control_msgs::msg::GripperCommand gripper_command_to_ros(const ::control_msgs::msg::v1::GripperCommand &bus) {
  ::control_msgs::msg::GripperCommand out;
  out.position = bus.position();
  out.max_effort = bus.max_effort();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsGripperCommandMapper
    : public TypedTopicMapper<ControlMsgsGripperCommandMapper, ::control_msgs::msg::GripperCommand> {
 public:
  const char *type_name() const override { return "control_msgs/msg/GripperCommand"; }

  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::GripperCommand &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::gripper_command_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::GripperCommand bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::GripperCommand bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::gripper_command_to_ros(bus);
  }
};
#else
struct ControlMsgsGripperCommandMapper : TopicMapper {
  const char *type_name() const override { return "control_msgs/msg/GripperCommand"; }
};
#endif

}  // namespace robot_bus
