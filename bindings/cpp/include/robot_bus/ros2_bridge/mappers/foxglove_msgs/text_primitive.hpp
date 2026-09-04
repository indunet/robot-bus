#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/text_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/text_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::TextPrimitive text_primitive_to_bus(const ::foxglove_msgs::msg::TextPrimitive &msg) {
  ::foxglove_msgs::msg::v1::TextPrimitive bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_billboard(msg.billboard);
  bus.set_font_size(msg.font_size);
  bus.set_scale_invariant(msg.scale_invariant);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  bus.set_text(msg.text.c_str());
  return bus;
}

inline ::foxglove_msgs::msg::TextPrimitive text_primitive_to_ros(const ::foxglove_msgs::msg::v1::TextPrimitive &bus) {
  ::foxglove_msgs::msg::TextPrimitive out;
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.billboard = bus.billboard();
  out.font_size = bus.font_size();
  out.scale_invariant = bus.scale_invariant();
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  out.text = bus.text();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsTextPrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsTextPrimitiveMapper, ::foxglove_msgs::msg::TextPrimitive> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::TextPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::text_primitive_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::TextPrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::TextPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::text_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsTextPrimitiveMapper : TopicMapper {};
#endif

}  // namespace robot_bus
