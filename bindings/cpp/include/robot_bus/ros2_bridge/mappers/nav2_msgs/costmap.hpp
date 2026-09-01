#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/costmap.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/nav2_msgs/costmap_meta_data.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/costmap.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::Costmap costmap_to_bus(const ::nav2_msgs::msg::Costmap &msg) {
  ::nav2_msgs::msg::v1::Costmap bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_metadata() = ::robot_bus::ros2_bridge_mappers::nav2_msgs::costmap_meta_data_to_bus(msg.metadata);
  {
    auto tmp = ::robot_bus::ros2_bridge_mappers::i8_seq_to_bytes(msg.data);
    bus.set_data(tmp.data(), tmp.size());
  }
  return bus;
}

inline ::nav2_msgs::msg::Costmap costmap_to_ros(const ::nav2_msgs::msg::v1::Costmap &bus) {
  ::nav2_msgs::msg::Costmap out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.metadata = ::robot_bus::ros2_bridge_mappers::nav2_msgs::costmap_meta_data_to_ros(bus.metadata());
  out.data = ::robot_bus::ros2_bridge_mappers::bytes_to_i8_seq(bus.data());
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsCostmapMapper
    : public TypedTopicMapper<Nav2MsgsCostmapMapper, ::nav2_msgs::msg::Costmap> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/Costmap"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::Costmap &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::costmap_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::Costmap bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::Costmap bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::costmap_to_ros(bus);
  }
};
#else
struct Nav2MsgsCostmapMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/Costmap"; }
};
#endif

}  // namespace robot_bus
