#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/drive_controller_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/mecanum_drive_controller_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::MecanumDriveControllerState mecanum_drive_controller_state_to_bus(const ::control_msgs::msg::MecanumDriveControllerState &msg) {
  ::control_msgs::msg::v1::MecanumDriveControllerState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_front_left_wheel_velocity(msg.front_left_wheel_velocity);
  bus.set_front_right_wheel_velocity(msg.front_right_wheel_velocity);
  bus.set_back_left_wheel_velocity(msg.back_left_wheel_velocity);
  bus.set_back_right_wheel_velocity(msg.back_right_wheel_velocity);
  *bus.mutable_reference_velocity() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(msg.reference_velocity);
  return bus;
}

inline ::control_msgs::msg::MecanumDriveControllerState mecanum_drive_controller_state_to_ros(const ::control_msgs::msg::v1::MecanumDriveControllerState &bus) {
  ::control_msgs::msg::MecanumDriveControllerState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.front_left_wheel_velocity = bus.front_left_wheel_velocity();
  out.front_right_wheel_velocity = bus.front_right_wheel_velocity();
  out.back_left_wheel_velocity = bus.back_left_wheel_velocity();
  out.back_right_wheel_velocity = bus.back_right_wheel_velocity();
  out.reference_velocity = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(bus.reference_velocity());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsMecanumDriveControllerStateMapper
    : public TypedTopicMapper<ControlMsgsMecanumDriveControllerStateMapper, ::control_msgs::msg::MecanumDriveControllerState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::MecanumDriveControllerState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::mecanum_drive_controller_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::MecanumDriveControllerState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::MecanumDriveControllerState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::mecanum_drive_controller_state_to_ros(bus);
  }
};
#else
struct ControlMsgsMecanumDriveControllerStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
