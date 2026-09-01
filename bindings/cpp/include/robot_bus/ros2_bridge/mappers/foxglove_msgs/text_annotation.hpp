#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/text_annotation.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/point2.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/text_annotation.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::TextAnnotation text_annotation_to_bus(const ::foxglove_msgs::msg::TextAnnotation &msg) {
  ::foxglove_msgs::msg::v1::TextAnnotation bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  *bus.mutable_position() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::point2_to_bus(msg.position);
  bus.set_text(msg.text.c_str());
  bus.set_font_size(msg.font_size);
  *bus.mutable_text_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.text_color);
  *bus.mutable_background_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.background_color);
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::TextAnnotation text_annotation_to_ros(const ::foxglove_msgs::msg::v1::TextAnnotation &bus) {
  ::foxglove_msgs::msg::TextAnnotation out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.position = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::point2_to_ros(bus.position());
  out.text = bus.text();
  out.font_size = bus.font_size();
  out.text_color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.text_color());
  out.background_color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.background_color());
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
class FoxgloveMsgsTextAnnotationMapper
    : public TypedTopicMapper<FoxgloveMsgsTextAnnotationMapper, ::foxglove_msgs::msg::TextAnnotation> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::TextAnnotation &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::text_annotation_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::TextAnnotation bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::TextAnnotation bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::text_annotation_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsTextAnnotationMapper : TopicMapper {};
#endif

}  // namespace robot_bus
