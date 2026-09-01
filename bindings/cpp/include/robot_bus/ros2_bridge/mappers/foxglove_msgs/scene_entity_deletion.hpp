#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/scene_entity_deletion.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/scene_entity_deletion.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::SceneEntityDeletion scene_entity_deletion_to_bus(const ::foxglove_msgs::msg::SceneEntityDeletion &msg) {
  ::foxglove_msgs::msg::v1::SceneEntityDeletion bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_type(static_cast<int32_t>(msg.type));
  bus.set_id(msg.id.c_str());
  return bus;
}

inline ::foxglove_msgs::msg::SceneEntityDeletion scene_entity_deletion_to_ros(const ::foxglove_msgs::msg::v1::SceneEntityDeletion &bus) {
  ::foxglove_msgs::msg::SceneEntityDeletion out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.type = bus.type();
  out.id = bus.id();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsSceneEntityDeletionMapper
    : public TypedTopicMapper<FoxgloveMsgsSceneEntityDeletionMapper, ::foxglove_msgs::msg::SceneEntityDeletion> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::SceneEntityDeletion &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::scene_entity_deletion_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::SceneEntityDeletion bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::SceneEntityDeletion bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::scene_entity_deletion_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsSceneEntityDeletionMapper : TopicMapper {};
#endif

}  // namespace robot_bus
