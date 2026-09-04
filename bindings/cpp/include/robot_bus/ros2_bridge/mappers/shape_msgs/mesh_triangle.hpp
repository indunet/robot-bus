#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/shape_msgs/msg/v1/mesh.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <shape_msgs/msg/mesh_triangle.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace shape_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::shape_msgs::msg::v1::MeshTriangle mesh_triangle_to_bus(const ::shape_msgs::msg::MeshTriangle &msg) {
  ::shape_msgs::msg::v1::MeshTriangle bus;
  for (auto x : msg.vertex_indices) {
    bus.add_vertex_indices(x);
  }
  return bus;
}

inline ::shape_msgs::msg::MeshTriangle mesh_triangle_to_ros(const ::shape_msgs::msg::v1::MeshTriangle &bus) {
  ::shape_msgs::msg::MeshTriangle out;
  out.vertex_indices.assign(bus.vertex_indices().begin(), bus.vertex_indices().end());
  return out;
}
#endif

}  // namespace shape_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ShapeMsgsMeshTriangleMapper
    : public TypedTopicMapper<ShapeMsgsMeshTriangleMapper, ::shape_msgs::msg::MeshTriangle> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::shape_msgs::msg::MeshTriangle &msg) const {
    auto bus = ros2_bridge_mappers::shape_msgs::mesh_triangle_to_bus(msg);
    return encode_pb(bus);
  }

  ::shape_msgs::msg::MeshTriangle bus_to_ros(BytesView payload) const {
    ::shape_msgs::msg::v1::MeshTriangle bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::shape_msgs::mesh_triangle_to_ros(bus);
  }
};
#else
struct ShapeMsgsMeshTriangleMapper : TopicMapper {};
#endif

}  // namespace robot_bus
