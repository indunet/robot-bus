#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/interactive_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/quaternion.hpp>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/marker.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/interactive_marker_control.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::InteractiveMarkerControl interactive_marker_control_to_bus(const ::visualization_msgs::msg::InteractiveMarkerControl &msg) {
  ::visualization_msgs::msg::v1::InteractiveMarkerControl bus;
  bus.set_name(msg.name.c_str());
  *bus.mutable_orientation() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_bus(msg.orientation);
  bus.set_orientation_mode(msg.orientation_mode);
  bus.set_interaction_mode(msg.interaction_mode);
  bus.set_always_visible(msg.always_visible);
  for (const auto &x : msg.markers) {
    *bus.add_markers() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::marker_to_bus(x);
  }
  bus.set_independent_marker_orientation(msg.independent_marker_orientation);
  bus.set_description(msg.description.c_str());
  return bus;
}

inline ::visualization_msgs::msg::InteractiveMarkerControl interactive_marker_control_to_ros(const ::visualization_msgs::msg::v1::InteractiveMarkerControl &bus) {
  ::visualization_msgs::msg::InteractiveMarkerControl out;
  out.name = bus.name();
  out.orientation = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_ros(bus.orientation());
  out.orientation_mode = bus.orientation_mode();
  out.interaction_mode = bus.interaction_mode();
  out.always_visible = bus.always_visible();
  out.markers.clear();
  for (const auto &x : bus.markers()) {
    out.markers.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::marker_to_ros(x));
  }
  out.independent_marker_orientation = bus.independent_marker_orientation();
  out.description = bus.description();
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsInteractiveMarkerControlMapper
    : public TypedTopicMapper<VisualizationMsgsInteractiveMarkerControlMapper, ::visualization_msgs::msg::InteractiveMarkerControl> {
 public:
  const char *type_name() const override { return "visualization_msgs/msg/InteractiveMarkerControl"; }

  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::InteractiveMarkerControl &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::interactive_marker_control_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::InteractiveMarkerControl bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::InteractiveMarkerControl bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::interactive_marker_control_to_ros(bus);
  }
};
#else
struct VisualizationMsgsInteractiveMarkerControlMapper : TopicMapper {
  const char *type_name() const override { return "visualization_msgs/msg/InteractiveMarkerControl"; }
};
#endif

}  // namespace robot_bus
