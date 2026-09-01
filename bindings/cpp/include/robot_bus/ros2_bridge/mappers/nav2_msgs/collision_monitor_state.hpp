#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/collision_monitor_state.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/collision_monitor_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::CollisionMonitorState collision_monitor_state_to_bus(const ::nav2_msgs::msg::CollisionMonitorState &msg) {
  ::nav2_msgs::msg::v1::CollisionMonitorState bus;
  bus.set_action_type(msg.action_type);
  bus.set_polygon_name(msg.polygon_name.c_str());
  return bus;
}

inline ::nav2_msgs::msg::CollisionMonitorState collision_monitor_state_to_ros(const ::nav2_msgs::msg::v1::CollisionMonitorState &bus) {
  ::nav2_msgs::msg::CollisionMonitorState out;
  out.action_type = bus.action_type();
  out.polygon_name = bus.polygon_name();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsCollisionMonitorStateMapper
    : public TypedTopicMapper<Nav2MsgsCollisionMonitorStateMapper, ::nav2_msgs::msg::CollisionMonitorState> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/CollisionMonitorState"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::CollisionMonitorState &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::collision_monitor_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::CollisionMonitorState bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::CollisionMonitorState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::collision_monitor_state_to_ros(bus);
  }
};
#else
struct Nav2MsgsCollisionMonitorStateMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/CollisionMonitorState"; }
};
#endif

}  // namespace robot_bus
