#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/costmap.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/costmap_filter_info.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::CostmapFilterInfo costmap_filter_info_to_bus(const ::nav2_msgs::msg::CostmapFilterInfo &msg) {
  ::nav2_msgs::msg::v1::CostmapFilterInfo bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_type(msg.type);
  bus.set_filter_mask_topic(msg.filter_mask_topic.c_str());
  bus.set_base(msg.base);
  bus.set_multiplier(msg.multiplier);
  return bus;
}

inline ::nav2_msgs::msg::CostmapFilterInfo costmap_filter_info_to_ros(const ::nav2_msgs::msg::v1::CostmapFilterInfo &bus) {
  ::nav2_msgs::msg::CostmapFilterInfo out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.type = bus.type();
  out.filter_mask_topic = bus.filter_mask_topic();
  out.base = bus.base();
  out.multiplier = bus.multiplier();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsCostmapFilterInfoMapper
    : public TypedTopicMapper<Nav2MsgsCostmapFilterInfoMapper, ::nav2_msgs::msg::CostmapFilterInfo> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::CostmapFilterInfo &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::costmap_filter_info_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::CostmapFilterInfo bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::CostmapFilterInfo bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::costmap_filter_info_to_ros(bus);
  }
};
#else
struct Nav2MsgsCostmapFilterInfoMapper : TopicMapper {};
#endif

}  // namespace robot_bus
