#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/route.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/route_node.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::RouteNode route_node_to_bus(const ::nav2_msgs::msg::RouteNode &msg) {
  ::nav2_msgs::msg::v1::RouteNode bus;
  bus.set_nodeid(msg.nodeid);
  *bus.mutable_position() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(msg.position);
  return bus;
}

inline ::nav2_msgs::msg::RouteNode route_node_to_ros(const ::nav2_msgs::msg::v1::RouteNode &bus) {
  ::nav2_msgs::msg::RouteNode out;
  out.nodeid = bus.nodeid();
  out.position = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(bus.position());
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsRouteNodeMapper
    : public TypedTopicMapper<Nav2MsgsRouteNodeMapper, ::nav2_msgs::msg::RouteNode> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::RouteNode &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::route_node_to_bus(msg);
    return encode_pb(bus);
  }

  ::nav2_msgs::msg::RouteNode bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::RouteNode bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::route_node_to_ros(bus);
  }
};
#else
struct Nav2MsgsRouteNodeMapper : TopicMapper {};
#endif

}  // namespace robot_bus
