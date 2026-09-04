#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/motion_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/control_msgs/motion_primitive.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/motion_primitive_sequence.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::MotionPrimitiveSequence motion_primitive_sequence_to_bus(const ::control_msgs::msg::MotionPrimitiveSequence &msg) {
  ::control_msgs::msg::v1::MotionPrimitiveSequence bus;
  for (const auto &x : msg.motions) {
    *bus.add_motions() = ::robot_bus::ros2_bridge_mappers::control_msgs::motion_primitive_to_bus(x);
  }
  return bus;
}

inline ::control_msgs::msg::MotionPrimitiveSequence motion_primitive_sequence_to_ros(const ::control_msgs::msg::v1::MotionPrimitiveSequence &bus) {
  ::control_msgs::msg::MotionPrimitiveSequence out;
  out.motions.clear();
  for (const auto &x : bus.motions()) {
    out.motions.push_back(::robot_bus::ros2_bridge_mappers::control_msgs::motion_primitive_to_ros(x));
  }
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsMotionPrimitiveSequenceMapper
    : public TypedTopicMapper<ControlMsgsMotionPrimitiveSequenceMapper, ::control_msgs::msg::MotionPrimitiveSequence> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::MotionPrimitiveSequence &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::motion_primitive_sequence_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::MotionPrimitiveSequence bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::MotionPrimitiveSequence bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::motion_primitive_sequence_to_ros(bus);
  }
};
#else
struct ControlMsgsMotionPrimitiveSequenceMapper : TopicMapper {};
#endif

}  // namespace robot_bus
