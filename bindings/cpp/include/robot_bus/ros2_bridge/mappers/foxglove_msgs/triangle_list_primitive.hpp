#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/triangle_list_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/point3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/triangle_list_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::TriangleListPrimitive triangle_list_primitive_to_bus(const ::foxglove_msgs::msg::TriangleListPrimitive &msg) {
  ::foxglove_msgs::msg::v1::TriangleListPrimitive bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::point3_to_bus(x);
  }
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  for (const auto &x : msg.colors) {
    *bus.add_colors() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(x);
  }
  for (auto x : msg.indices) {
    bus.add_indices(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::TriangleListPrimitive triangle_list_primitive_to_ros(const ::foxglove_msgs::msg::v1::TriangleListPrimitive &bus) {
  ::foxglove_msgs::msg::TriangleListPrimitive out;
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::point3_to_ros(x));
  }
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  out.colors.clear();
  for (const auto &x : bus.colors()) {
    out.colors.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(x));
  }
  out.indices.assign(bus.indices().begin(), bus.indices().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsTriangleListPrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsTriangleListPrimitiveMapper, ::foxglove_msgs::msg::TriangleListPrimitive> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/TriangleListPrimitive"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::TriangleListPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::triangle_list_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::TriangleListPrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::TriangleListPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::triangle_list_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsTriangleListPrimitiveMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/TriangleListPrimitive"; }
};
#endif

}  // namespace robot_bus
