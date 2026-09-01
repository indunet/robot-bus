#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/image_annotations.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/circle_annotation.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/points_annotation.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/text_annotation.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/image_annotations.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::ImageAnnotations image_annotations_to_bus(const ::foxglove_msgs::msg::ImageAnnotations &msg) {
  ::foxglove_msgs::msg::v1::ImageAnnotations bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  for (const auto &x : msg.circles) {
    *bus.add_circles() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::circle_annotation_to_bus(x);
  }
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::points_annotation_to_bus(x);
  }
  for (const auto &x : msg.texts) {
    *bus.add_texts() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::text_annotation_to_bus(x);
  }
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::ImageAnnotations image_annotations_to_ros(const ::foxglove_msgs::msg::v1::ImageAnnotations &bus) {
  ::foxglove_msgs::msg::ImageAnnotations out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.circles.clear();
  for (const auto &x : bus.circles()) {
    out.circles.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::circle_annotation_to_ros(x));
  }
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::points_annotation_to_ros(x));
  }
  out.texts.clear();
  for (const auto &x : bus.texts()) {
    out.texts.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::text_annotation_to_ros(x));
  }
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
class FoxgloveMsgsImageAnnotationsMapper
    : public TypedTopicMapper<FoxgloveMsgsImageAnnotationsMapper, ::foxglove_msgs::msg::ImageAnnotations> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/ImageAnnotations"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::ImageAnnotations &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::image_annotations_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::ImageAnnotations bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::ImageAnnotations bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::image_annotations_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsImageAnnotationsMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/ImageAnnotations"; }
};
#endif

}  // namespace robot_bus
