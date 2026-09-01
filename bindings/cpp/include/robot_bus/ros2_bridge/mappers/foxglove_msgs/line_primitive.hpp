#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/line_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/point3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/line_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::LinePrimitive line_primitive_to_bus(const ::foxglove_msgs::msg::LinePrimitive &msg) {
  ::foxglove_msgs::msg::v1::LinePrimitive bus;
  bus.set_type(static_cast<int32_t>(msg.type));
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_thickness(msg.thickness);
  bus.set_scale_invariant(msg.scale_invariant);
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

inline ::foxglove_msgs::msg::LinePrimitive line_primitive_to_ros(const ::foxglove_msgs::msg::v1::LinePrimitive &bus) {
  ::foxglove_msgs::msg::LinePrimitive out;
  out.type = bus.type();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.thickness = bus.thickness();
  out.scale_invariant = bus.scale_invariant();
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
class FoxgloveMsgsLinePrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsLinePrimitiveMapper, ::foxglove_msgs::msg::LinePrimitive> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/LinePrimitive"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::LinePrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::line_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::LinePrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::LinePrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::line_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsLinePrimitiveMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/LinePrimitive"; }
};
#endif

}  // namespace robot_bus
