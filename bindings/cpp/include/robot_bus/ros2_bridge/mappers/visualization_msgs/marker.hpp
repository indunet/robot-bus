#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/std_msgs/color_rgba.hpp>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/duration.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/compressed_image.hpp>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/uv_coordinate.hpp>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/mesh_file.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/marker.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::Marker marker_to_bus(const ::visualization_msgs::msg::Marker &msg) {
  ::visualization_msgs::msg::v1::Marker bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_ns(msg.ns.c_str());
  bus.set_id(msg.id);
  bus.set_type(msg.type);
  bus.set_action(msg.action);
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  *bus.mutable_scale() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.scale);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_bus(msg.color);
  *bus.mutable_lifetime() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_bus(msg.lifetime);
  bus.set_frame_locked(msg.frame_locked);
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(x);
  }
  for (const auto &x : msg.colors) {
    *bus.add_colors() = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_bus(x);
  }
  bus.set_texture_resource(msg.texture_resource.c_str());
  *bus.mutable_texture() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::compressed_image_to_bus(msg.texture);
  for (const auto &x : msg.uv_coordinates) {
    *bus.add_uv_coordinates() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::uv_coordinate_to_bus(x);
  }
  bus.set_text(msg.text.c_str());
  bus.set_mesh_resource(msg.mesh_resource.c_str());
  *bus.mutable_mesh_file() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::mesh_file_to_bus(msg.mesh_file);
  bus.set_mesh_use_embedded_materials(msg.mesh_use_embedded_materials);
  return bus;
}

inline ::visualization_msgs::msg::Marker marker_to_ros(const ::visualization_msgs::msg::v1::Marker &bus) {
  ::visualization_msgs::msg::Marker out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.ns = bus.ns();
  out.id = bus.id();
  out.type = bus.type();
  out.action = bus.action();
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  out.scale = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.scale());
  out.color = ::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_ros(bus.color());
  out.lifetime = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_ros(bus.lifetime());
  out.frame_locked = bus.frame_locked();
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(x));
  }
  out.colors.clear();
  for (const auto &x : bus.colors()) {
    out.colors.push_back(::robot_bus::ros2_bridge_mappers::std_msgs::color_rgba_to_ros(x));
  }
  out.texture_resource = bus.texture_resource();
  out.texture = ::robot_bus::ros2_bridge_mappers::sensor_msgs::compressed_image_to_ros(bus.texture());
  out.uv_coordinates.clear();
  for (const auto &x : bus.uv_coordinates()) {
    out.uv_coordinates.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::uv_coordinate_to_ros(x));
  }
  out.text = bus.text();
  out.mesh_resource = bus.mesh_resource();
  out.mesh_file = ::robot_bus::ros2_bridge_mappers::visualization_msgs::mesh_file_to_ros(bus.mesh_file());
  out.mesh_use_embedded_materials = bus.mesh_use_embedded_materials();
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsMarkerMapper
    : public TypedTopicMapper<VisualizationMsgsMarkerMapper, ::visualization_msgs::msg::Marker> {
 public:
  const char *type_name() const override { return "visualization_msgs/msg/Marker"; }

  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::Marker &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::marker_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::Marker bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::Marker bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::marker_to_ros(bus);
  }
};
#else
struct VisualizationMsgsMarkerMapper : TopicMapper {
  const char *type_name() const override { return "visualization_msgs/msg/Marker"; }
};
#endif

}  // namespace robot_bus
