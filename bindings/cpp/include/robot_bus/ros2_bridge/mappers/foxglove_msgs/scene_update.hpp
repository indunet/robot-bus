#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/scene_update.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/scene_entity_deletion.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/scene_entity.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/scene_update.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::SceneUpdate scene_update_to_bus(const ::foxglove_msgs::msg::SceneUpdate &msg) {
  ::foxglove_msgs::msg::v1::SceneUpdate bus;
  for (const auto &x : msg.deletions) {
    *bus.add_deletions() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::scene_entity_deletion_to_bus(x);
  }
  for (const auto &x : msg.entities) {
    *bus.add_entities() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::scene_entity_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::SceneUpdate scene_update_to_ros(const ::foxglove_msgs::msg::v1::SceneUpdate &bus) {
  ::foxglove_msgs::msg::SceneUpdate out;
  out.deletions.clear();
  for (const auto &x : bus.deletions()) {
    out.deletions.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::scene_entity_deletion_to_ros(x));
  }
  out.entities.clear();
  for (const auto &x : bus.entities()) {
    out.entities.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::scene_entity_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsSceneUpdateMapper
    : public TypedTopicMapper<FoxgloveMsgsSceneUpdateMapper, ::foxglove_msgs::msg::SceneUpdate> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/SceneUpdate"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::SceneUpdate &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::scene_update_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::SceneUpdate bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::SceneUpdate bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::scene_update_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsSceneUpdateMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/SceneUpdate"; }
};
#endif

}  // namespace robot_bus
