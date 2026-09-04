#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/location_fixes.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/location_fix.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/location_fixes.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::LocationFixes location_fixes_to_bus(const ::foxglove_msgs::msg::LocationFixes &msg) {
  ::foxglove_msgs::msg::v1::LocationFixes bus;
  for (const auto &x : msg.fixes) {
    *bus.add_fixes() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::location_fix_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::LocationFixes location_fixes_to_ros(const ::foxglove_msgs::msg::v1::LocationFixes &bus) {
  ::foxglove_msgs::msg::LocationFixes out;
  out.fixes.clear();
  for (const auto &x : bus.fixes()) {
    out.fixes.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::location_fix_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsLocationFixesMapper
    : public TypedTopicMapper<FoxgloveMsgsLocationFixesMapper, ::foxglove_msgs::msg::LocationFixes> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::LocationFixes &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::location_fixes_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::LocationFixes bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::LocationFixes bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::location_fixes_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsLocationFixesMapper : TopicMapper {};
#endif

}  // namespace robot_bus
