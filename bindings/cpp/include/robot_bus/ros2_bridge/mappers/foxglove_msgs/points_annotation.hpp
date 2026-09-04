#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/points_annotation.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/point2.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/points_annotation.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::PointsAnnotation points_annotation_to_bus(const ::foxglove_msgs::msg::PointsAnnotation &msg) {
  ::foxglove_msgs::msg::v1::PointsAnnotation bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_type(static_cast<int32_t>(msg.type));
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::point2_to_bus(x);
  }
  *bus.mutable_outline_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.outline_color);
  for (const auto &x : msg.outline_colors) {
    *bus.add_outline_colors() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(x);
  }
  *bus.mutable_fill_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.fill_color);
  bus.set_thickness(msg.thickness);
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::PointsAnnotation points_annotation_to_ros(const ::foxglove_msgs::msg::v1::PointsAnnotation &bus) {
  ::foxglove_msgs::msg::PointsAnnotation out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.type = bus.type();
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::point2_to_ros(x));
  }
  out.outline_color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.outline_color());
  out.outline_colors.clear();
  for (const auto &x : bus.outline_colors()) {
    out.outline_colors.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(x));
  }
  out.fill_color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.fill_color());
  out.thickness = bus.thickness();
  out.metadata.clear();
  for (const auto &x : bus.metadata()) {
    out.metadata.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPointsAnnotationMapper
    : public TypedTopicMapper<FoxgloveMsgsPointsAnnotationMapper, ::foxglove_msgs::msg::PointsAnnotation> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::PointsAnnotation &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::points_annotation_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::PointsAnnotation bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::PointsAnnotation bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::points_annotation_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPointsAnnotationMapper : TopicMapper {};
#endif

}  // namespace robot_bus
