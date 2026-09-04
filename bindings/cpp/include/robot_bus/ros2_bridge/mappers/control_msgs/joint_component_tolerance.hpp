#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/joint_component_tolerance.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/joint_component_tolerance.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::JointComponentTolerance joint_component_tolerance_to_bus(const ::control_msgs::msg::JointComponentTolerance &msg) {
  ::control_msgs::msg::v1::JointComponentTolerance bus;
  bus.set_joint_name(msg.joint_name.c_str());
  bus.set_component(msg.component);
  bus.set_value(msg.value);
  return bus;
}

inline ::control_msgs::msg::JointComponentTolerance joint_component_tolerance_to_ros(const ::control_msgs::msg::v1::JointComponentTolerance &bus) {
  ::control_msgs::msg::JointComponentTolerance out;
  out.joint_name = bus.joint_name();
  out.component = bus.component();
  out.value = bus.value();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsJointComponentToleranceMapper
    : public TypedTopicMapper<ControlMsgsJointComponentToleranceMapper, ::control_msgs::msg::JointComponentTolerance> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::JointComponentTolerance &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::joint_component_tolerance_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::JointComponentTolerance bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::JointComponentTolerance bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::joint_component_tolerance_to_ros(bus);
  }
};
#else
struct ControlMsgsJointComponentToleranceMapper : TopicMapper {};
#endif

}  // namespace robot_bus
