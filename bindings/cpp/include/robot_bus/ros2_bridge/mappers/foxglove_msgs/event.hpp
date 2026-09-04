#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/event.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/event.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Event event_to_bus(const ::foxglove_msgs::msg::Event &msg) {
  ::foxglove_msgs::msg::v1::Event bus;
  *bus.mutable_start_time() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.start_time);
  *bus.mutable_end_time() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.end_time);
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::Event event_to_ros(const ::foxglove_msgs::msg::v1::Event &bus) {
  ::foxglove_msgs::msg::Event out;
  out.start_time = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.start_time());
  out.end_time = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.end_time());
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
class FoxgloveMsgsEventMapper
    : public TypedTopicMapper<FoxgloveMsgsEventMapper, ::foxglove_msgs::msg::Event> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Event &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::event_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::Event bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Event bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::event_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsEventMapper : TopicMapper {};
#endif

}  // namespace robot_bus
