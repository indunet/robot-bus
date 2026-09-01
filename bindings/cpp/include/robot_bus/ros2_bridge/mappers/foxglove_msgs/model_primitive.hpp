#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/model_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/model_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::ModelPrimitive model_primitive_to_bus(const ::foxglove_msgs::msg::ModelPrimitive &msg) {
  ::foxglove_msgs::msg::v1::ModelPrimitive bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  *bus.mutable_scale() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.scale);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  bus.set_override_color(msg.override_color);
  bus.set_url(msg.url.c_str());
  bus.set_media_type(msg.media_type.c_str());
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::foxglove_msgs::msg::ModelPrimitive model_primitive_to_ros(const ::foxglove_msgs::msg::v1::ModelPrimitive &bus) {
  ::foxglove_msgs::msg::ModelPrimitive out;
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.scale = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.scale());
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  out.override_color = bus.override_color();
  out.url = bus.url();
  out.media_type = bus.media_type();
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsModelPrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsModelPrimitiveMapper, ::foxglove_msgs::msg::ModelPrimitive> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::ModelPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::model_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::ModelPrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::ModelPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::model_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsModelPrimitiveMapper : TopicMapper {};
#endif

}  // namespace robot_bus
