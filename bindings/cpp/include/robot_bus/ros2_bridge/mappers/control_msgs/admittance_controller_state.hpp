#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/admittance_controller_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/transform_stamped.hpp>
#include <robot_bus/ros2_bridge/mappers/std_msgs/float64_multi_array.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist_stamped.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/wrench_stamped.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/joint_state.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/admittance_controller_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::AdmittanceControllerState admittance_controller_state_to_bus(const ::control_msgs::msg::AdmittanceControllerState &msg) {
  ::control_msgs::msg::v1::AdmittanceControllerState bus;
  *bus.mutable_ref_trans_base_fts() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_bus(msg.ref_trans_base_fts);
  *bus.mutable_selected_axes() = ::robot_bus::ros2_bridge_mappers::std_msgs::float64_multi_array_to_bus(msg.selected_axes);
  *bus.mutable_ft_sensor_frame() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_bus(msg.ft_sensor_frame);
  *bus.mutable_admittance_position() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_bus(msg.admittance_position);
  *bus.mutable_admittance_acceleration() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_stamped_to_bus(msg.admittance_acceleration);
  *bus.mutable_admittance_velocity() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_stamped_to_bus(msg.admittance_velocity);
  *bus.mutable_wrench_base() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_stamped_to_bus(msg.wrench_base);
  *bus.mutable_robot_ref_trans_base_fts() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_bus(msg.robot_ref_trans_base_fts);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  *bus.mutable_joint_state() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::joint_state_to_bus(msg.joint_state);
  return bus;
}

inline ::control_msgs::msg::AdmittanceControllerState admittance_controller_state_to_ros(const ::control_msgs::msg::v1::AdmittanceControllerState &bus) {
  ::control_msgs::msg::AdmittanceControllerState out;
  out.ref_trans_base_fts = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_ros(bus.ref_trans_base_fts());
  out.selected_axes = ::robot_bus::ros2_bridge_mappers::std_msgs::float64_multi_array_to_ros(bus.selected_axes());
  out.ft_sensor_frame = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_ros(bus.ft_sensor_frame());
  out.admittance_position = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_ros(bus.admittance_position());
  out.admittance_acceleration = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_stamped_to_ros(bus.admittance_acceleration());
  out.admittance_velocity = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_stamped_to_ros(bus.admittance_velocity());
  out.wrench_base = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_stamped_to_ros(bus.wrench_base());
  out.robot_ref_trans_base_fts = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_ros(bus.robot_ref_trans_base_fts());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
  }
  out.joint_state = ::robot_bus::ros2_bridge_mappers::sensor_msgs::joint_state_to_ros(bus.joint_state());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsAdmittanceControllerStateMapper
    : public TypedTopicMapper<ControlMsgsAdmittanceControllerStateMapper, ::control_msgs::msg::AdmittanceControllerState> {
 public:
  const char *type_name() const override { return "control_msgs/msg/AdmittanceControllerState"; }

  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::AdmittanceControllerState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::admittance_controller_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::AdmittanceControllerState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::AdmittanceControllerState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::admittance_controller_state_to_ros(bus);
  }
};
#else
struct ControlMsgsAdmittanceControllerStateMapper : TopicMapper {
  const char *type_name() const override { return "control_msgs/msg/AdmittanceControllerState"; }
};
#endif

}  // namespace robot_bus
