#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/route.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/route_edge.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::RouteEdge route_edge_to_bus(const ::nav2_msgs::msg::RouteEdge &msg) {
  ::nav2_msgs::msg::v1::RouteEdge bus;
  bus.set_edgeid(msg.edgeid);
  bus.set_start(msg.start);
  bus.set_end(msg.end);
  return bus;
}

inline ::nav2_msgs::msg::RouteEdge route_edge_to_ros(const ::nav2_msgs::msg::v1::RouteEdge &bus) {
  ::nav2_msgs::msg::RouteEdge out;
  out.edgeid = bus.edgeid();
  out.start = bus.start();
  out.end = bus.end();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsRouteEdgeMapper
    : public TypedTopicMapper<Nav2MsgsRouteEdgeMapper, ::nav2_msgs::msg::RouteEdge> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/RouteEdge"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::RouteEdge &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::route_edge_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::RouteEdge bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::RouteEdge bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::route_edge_to_ros(bus);
  }
};
#else
struct Nav2MsgsRouteEdgeMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/RouteEdge"; }
};
#endif

}  // namespace robot_bus
