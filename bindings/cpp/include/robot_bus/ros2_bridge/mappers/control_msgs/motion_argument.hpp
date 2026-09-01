#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/motion_primitive.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/motion_argument.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::MotionArgument motion_argument_to_bus(const ::control_msgs::msg::MotionArgument &msg) {
  ::control_msgs::msg::v1::MotionArgument bus;
  bus.set_name(msg.name.c_str());
  bus.set_value(msg.value);
  return bus;
}

inline ::control_msgs::msg::MotionArgument motion_argument_to_ros(const ::control_msgs::msg::v1::MotionArgument &bus) {
  ::control_msgs::msg::MotionArgument out;
  out.name = bus.name();
  out.value = bus.value();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsMotionArgumentMapper
    : public TypedTopicMapper<ControlMsgsMotionArgumentMapper, ::control_msgs::msg::MotionArgument> {
 public:
  const char *type_name() const override { return "control_msgs/msg/MotionArgument"; }

  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::MotionArgument &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::motion_argument_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::MotionArgument bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::MotionArgument bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::motion_argument_to_ros(bus);
  }
};
#else
struct ControlMsgsMotionArgumentMapper : TopicMapper {
  const char *type_name() const override { return "control_msgs/msg/MotionArgument"; }
};
#endif

}  // namespace robot_bus
