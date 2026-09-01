#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/joint_tolerance.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/joint_tolerance.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::JointTolerance joint_tolerance_to_bus(const ::control_msgs::msg::JointTolerance &msg) {
  ::control_msgs::msg::v1::JointTolerance bus;
  bus.set_name(msg.name.c_str());
  bus.set_position(msg.position);
  bus.set_velocity(msg.velocity);
  bus.set_acceleration(msg.acceleration);
  return bus;
}

inline ::control_msgs::msg::JointTolerance joint_tolerance_to_ros(const ::control_msgs::msg::v1::JointTolerance &bus) {
  ::control_msgs::msg::JointTolerance out;
  out.name = bus.name();
  out.position = bus.position();
  out.velocity = bus.velocity();
  out.acceleration = bus.acceleration();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsJointToleranceMapper
    : public TypedTopicMapper<ControlMsgsJointToleranceMapper, ::control_msgs::msg::JointTolerance> {
 public:
  const char *type_name() const override { return "control_msgs/msg/JointTolerance"; }

  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::JointTolerance &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::joint_tolerance_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::JointTolerance bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::JointTolerance bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::joint_tolerance_to_ros(bus);
  }
};
#else
struct ControlMsgsJointToleranceMapper : TopicMapper {
  const char *type_name() const override { return "control_msgs/msg/JointTolerance"; }
};
#endif

}  // namespace robot_bus
