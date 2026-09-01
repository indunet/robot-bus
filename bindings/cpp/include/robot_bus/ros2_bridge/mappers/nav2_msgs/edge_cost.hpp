#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/route.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/edge_cost.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::EdgeCost edge_cost_to_bus(const ::nav2_msgs::msg::EdgeCost &msg) {
  ::nav2_msgs::msg::v1::EdgeCost bus;
  bus.set_edgeid(msg.edgeid);
  bus.set_cost(msg.cost);
  return bus;
}

inline ::nav2_msgs::msg::EdgeCost edge_cost_to_ros(const ::nav2_msgs::msg::v1::EdgeCost &bus) {
  ::nav2_msgs::msg::EdgeCost out;
  out.edgeid = bus.edgeid();
  out.cost = bus.cost();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsEdgeCostMapper
    : public TypedTopicMapper<Nav2MsgsEdgeCostMapper, ::nav2_msgs::msg::EdgeCost> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/EdgeCost"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::EdgeCost &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::edge_cost_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::EdgeCost bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::EdgeCost bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::edge_cost_to_ros(bus);
  }
};
#else
struct Nav2MsgsEdgeCostMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/EdgeCost"; }
};
#endif

}  // namespace robot_bus
