#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/image_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>
#include <robot_bus/ros2_bridge/mappers/std_msgs/color_rgba.hpp>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/duration.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/image_marker.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::ImageMarker image_marker_to_bus(const ::visualization_msgs::msg::ImageMarker &msg) {
  ::visualization_msgs::msg::v1::ImageMarker bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_ns(msg.ns.c_str());
  bus.set_id(msg.id);
  bus.set_type(msg.type);
  bus.set_action(msg.action);
  *bus.mutable_position() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(msg.position);
  bus.set_scale(msg.scale);
  *bus.mutable_outline_color() = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_bus(msg.outline_color);
  bus.set_filled(msg.filled);
  *bus.mutable_fill_color() = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_bus(msg.fill_color);
  *bus.mutable_lifetime() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_bus(msg.lifetime);
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(x);
  }
  for (const auto &x : msg.outline_colors) {
    *bus.add_outline_colors() = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_bus(x);
  }
  return bus;
}

inline ::visualization_msgs::msg::ImageMarker image_marker_to_ros(const ::visualization_msgs::msg::v1::ImageMarker &bus) {
  ::visualization_msgs::msg::ImageMarker out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.ns = bus.ns();
  out.id = bus.id();
  out.type = bus.type();
  out.action = bus.action();
  out.position = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(bus.position());
  out.scale = bus.scale();
  out.outline_color = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_ros(bus.outline_color());
  out.filled = bus.filled();
  out.fill_color = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_ros(bus.fill_color());
  out.lifetime = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_ros(bus.lifetime());
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(x));
  }
  out.outline_colors.clear();
  for (const auto &x : bus.outline_colors()) {
    out.outline_colors.push_back(::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_ros(x));
  }
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsImageMarkerMapper
    : public TypedTopicMapper<VisualizationMsgsImageMarkerMapper, ::visualization_msgs::msg::ImageMarker> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::ImageMarker &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::image_marker_to_bus(msg);
    return encode_pb(bus);
  }

  ::visualization_msgs::msg::ImageMarker bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::ImageMarker bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::image_marker_to_ros(bus);
  }
};
#else
struct VisualizationMsgsImageMarkerMapper : TopicMapper {};
#endif

}  // namespace robot_bus
