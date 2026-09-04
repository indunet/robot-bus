#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/shape_msgs/msg/v1/mesh.pb.h>
#include <robot_bus/ros2_bridge/mappers/shape_msgs/mesh_triangle.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <shape_msgs/msg/mesh.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace shape_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::shape_msgs::msg::v1::Mesh mesh_to_bus(const ::shape_msgs::msg::Mesh &msg) {
  ::shape_msgs::msg::v1::Mesh bus;
  for (const auto &x : msg.triangles) {
    *bus.add_triangles() = ::robot_bus::ros2_bridge_mappers::shape_msgs::mesh_triangle_to_bus(x);
  }
  for (const auto &x : msg.vertices) {
    *bus.add_vertices() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(x);
  }
  return bus;
}

inline ::shape_msgs::msg::Mesh mesh_to_ros(const ::shape_msgs::msg::v1::Mesh &bus) {
  ::shape_msgs::msg::Mesh out;
  out.triangles.clear();
  for (const auto &x : bus.triangles()) {
    out.triangles.push_back(::robot_bus::ros2_bridge_mappers::shape_msgs::mesh_triangle_to_ros(x));
  }
  out.vertices.clear();
  for (const auto &x : bus.vertices()) {
    out.vertices.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(x));
  }
  return out;
}
#endif

}  // namespace shape_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ShapeMsgsMeshMapper
    : public TypedTopicMapper<ShapeMsgsMeshMapper, ::shape_msgs::msg::Mesh> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::shape_msgs::msg::Mesh &msg) const {
    auto bus = ros2_bridge_mappers::shape_msgs::mesh_to_bus(msg);
    return encode_pb(bus);
  }

  ::shape_msgs::msg::Mesh bus_to_ros(BytesView payload) const {
    ::shape_msgs::msg::v1::Mesh bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::shape_msgs::mesh_to_ros(bus);
  }
};
#else
struct ShapeMsgsMeshMapper : TopicMapper {};
#endif

}  // namespace robot_bus
