#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/route.pb.h>
#include <robot_bus/ros2_bridge/mappers/nav2_msgs/route_node.hpp>
#include <robot_bus/ros2_bridge/mappers/nav2_msgs/route_edge.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/route.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::Route route_to_bus(const ::nav2_msgs::msg::Route &msg) {
  ::nav2_msgs::msg::v1::Route bus;
  for (const auto &x : msg.nodes) {
    *bus.add_nodes() = ::robot_bus::ros2_bridge_mappers::nav2_msgs::route_node_to_bus(x);
  }
  for (const auto &x : msg.edges) {
    *bus.add_edges() = ::robot_bus::ros2_bridge_mappers::nav2_msgs::route_edge_to_bus(x);
  }
  return bus;
}

inline ::nav2_msgs::msg::Route route_to_ros(const ::nav2_msgs::msg::v1::Route &bus) {
  ::nav2_msgs::msg::Route out;
  out.nodes.clear();
  for (const auto &x : bus.nodes()) {
    out.nodes.push_back(::robot_bus::ros2_bridge_mappers::nav2_msgs::route_node_to_ros(x));
  }
  out.edges.clear();
  for (const auto &x : bus.edges()) {
    out.edges.push_back(::robot_bus::ros2_bridge_mappers::nav2_msgs::route_edge_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsRouteMapper
    : public TypedTopicMapper<Nav2MsgsRouteMapper, ::nav2_msgs::msg::Route> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::Route &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::route_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::Route bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::Route bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::route_to_ros(bus);
  }
};
#else
struct Nav2MsgsRouteMapper : TopicMapper {};
#endif

}  // namespace robot_bus
