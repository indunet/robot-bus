#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/key_value_pair.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/key_value_pair.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::KeyValuePair key_value_pair_to_bus(const ::foxglove_msgs::msg::KeyValuePair &msg) {
  ::foxglove_msgs::msg::v1::KeyValuePair bus;
  bus.set_key(msg.key.c_str());
  bus.set_value(msg.value.c_str());
  return bus;
}

inline ::foxglove_msgs::msg::KeyValuePair key_value_pair_to_ros(const ::foxglove_msgs::msg::v1::KeyValuePair &bus) {
  ::foxglove_msgs::msg::KeyValuePair out;
  out.key = bus.key();
  out.value = bus.value();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsKeyValuePairMapper
    : public TypedTopicMapper<FoxgloveMsgsKeyValuePairMapper, ::foxglove_msgs::msg::KeyValuePair> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::KeyValuePair &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::KeyValuePair bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::KeyValuePair bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsKeyValuePairMapper : TopicMapper {};
#endif

}  // namespace robot_bus
