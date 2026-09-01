#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/drive_controller_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/steering_controller_status.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::SteeringControllerStatus steering_controller_status_to_bus(const ::control_msgs::msg::SteeringControllerStatus &msg) {
  ::control_msgs::msg::v1::SteeringControllerStatus bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (auto x : msg.traction_wheels_position) {
    bus.add_traction_wheels_position(x);
  }
  for (auto x : msg.traction_wheels_velocity) {
    bus.add_traction_wheels_velocity(x);
  }
  for (auto x : msg.steer_positions) {
    bus.add_steer_positions(x);
  }
  for (auto x : msg.linear_velocity_command) {
    bus.add_linear_velocity_command(x);
  }
  for (auto x : msg.steering_angle_command) {
    bus.add_steering_angle_command(x);
  }
  return bus;
}

inline ::control_msgs::msg::SteeringControllerStatus steering_controller_status_to_ros(const ::control_msgs::msg::v1::SteeringControllerStatus &bus) {
  ::control_msgs::msg::SteeringControllerStatus out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.traction_wheels_position.assign(bus.traction_wheels_position().begin(), bus.traction_wheels_position().end());
  out.traction_wheels_velocity.assign(bus.traction_wheels_velocity().begin(), bus.traction_wheels_velocity().end());
  out.steer_positions.assign(bus.steer_positions().begin(), bus.steer_positions().end());
  out.linear_velocity_command.assign(bus.linear_velocity_command().begin(), bus.linear_velocity_command().end());
  out.steering_angle_command.assign(bus.steering_angle_command().begin(), bus.steering_angle_command().end());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsSteeringControllerStatusMapper
    : public TypedTopicMapper<ControlMsgsSteeringControllerStatusMapper, ::control_msgs::msg::SteeringControllerStatus> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::SteeringControllerStatus &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::steering_controller_status_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::SteeringControllerStatus bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::SteeringControllerStatus bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::steering_controller_status_to_ros(bus);
  }
};
#else
struct ControlMsgsSteeringControllerStatusMapper : TopicMapper {};
#endif

}  // namespace robot_bus
