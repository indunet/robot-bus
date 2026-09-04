#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/interactive_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/menu_entry.hpp>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/interactive_marker_control.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/interactive_marker.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::InteractiveMarker interactive_marker_to_bus(const ::visualization_msgs::msg::InteractiveMarker &msg) {
  ::visualization_msgs::msg::v1::InteractiveMarker bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  bus.set_name(msg.name.c_str());
  bus.set_description(msg.description.c_str());
  bus.set_scale(msg.scale);
  for (const auto &x : msg.menu_entries) {
    *bus.add_menu_entries() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::menu_entry_to_bus(x);
  }
  for (const auto &x : msg.controls) {
    *bus.add_controls() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_control_to_bus(x);
  }
  return bus;
}

inline ::visualization_msgs::msg::InteractiveMarker interactive_marker_to_ros(const ::visualization_msgs::msg::v1::InteractiveMarker &bus) {
  ::visualization_msgs::msg::InteractiveMarker out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  out.name = bus.name();
  out.description = bus.description();
  out.scale = bus.scale();
  out.menu_entries.clear();
  for (const auto &x : bus.menu_entries()) {
    out.menu_entries.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::menu_entry_to_ros(x));
  }
  out.controls.clear();
  for (const auto &x : bus.controls()) {
    out.controls.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_control_to_ros(x));
  }
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsInteractiveMarkerMapper
    : public TypedTopicMapper<VisualizationMsgsInteractiveMarkerMapper, ::visualization_msgs::msg::InteractiveMarker> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::InteractiveMarker &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::interactive_marker_to_bus(msg);
    return encode_pb(bus);
  }

  ::visualization_msgs::msg::InteractiveMarker bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::InteractiveMarker bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::interactive_marker_to_ros(bus);
  }
};
#else
struct VisualizationMsgsInteractiveMarkerMapper : TopicMapper {};
#endif

}  // namespace robot_bus
