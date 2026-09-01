#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/motion_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/control_msgs/motion_argument.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose_stamped.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/motion_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::MotionPrimitive motion_primitive_to_bus(const ::control_msgs::msg::MotionPrimitive &msg) {
  ::control_msgs::msg::v1::MotionPrimitive bus;
  bus.set_type(msg.type);
  bus.set_blend_radius(msg.blend_radius);
  for (const auto &x : msg.additional_arguments) {
    *bus.add_additional_arguments() = ::robot_bus::ros2_bridge_mappers::control_msgs::motion_argument_to_bus(x);
  }
  for (const auto &x : msg.poses) {
    *bus.add_poses() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_bus(x);
  }
  for (auto x : msg.joint_positions) {
    bus.add_joint_positions(x);
  }
  return bus;
}

inline ::control_msgs::msg::MotionPrimitive motion_primitive_to_ros(const ::control_msgs::msg::v1::MotionPrimitive &bus) {
  ::control_msgs::msg::MotionPrimitive out;
  out.type = bus.type();
  out.blend_radius = bus.blend_radius();
  out.additional_arguments.clear();
  for (const auto &x : bus.additional_arguments()) {
    out.additional_arguments.push_back(::robot_bus::ros2_bridge_mappers::control_msgs::motion_argument_to_ros(x));
  }
  out.poses.clear();
  for (const auto &x : bus.poses()) {
    out.poses.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_ros(x));
  }
  out.joint_positions.assign(bus.joint_positions().begin(), bus.joint_positions().end());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsMotionPrimitiveMapper
    : public TypedTopicMapper<ControlMsgsMotionPrimitiveMapper, ::control_msgs::msg::MotionPrimitive> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::MotionPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::motion_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::MotionPrimitive bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::MotionPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::motion_primitive_to_ros(bus);
  }
};
#else
struct ControlMsgsMotionPrimitiveMapper : TopicMapper {};
#endif

}  // namespace robot_bus
