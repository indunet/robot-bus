#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/speed_limit.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/speed_limit.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::SpeedLimit speed_limit_to_bus(const ::nav2_msgs::msg::SpeedLimit &msg) {
  ::nav2_msgs::msg::v1::SpeedLimit bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_percentage(msg.percentage);
  bus.set_speed_limit(msg.speed_limit);
  return bus;
}

inline ::nav2_msgs::msg::SpeedLimit speed_limit_to_ros(const ::nav2_msgs::msg::v1::SpeedLimit &bus) {
  ::nav2_msgs::msg::SpeedLimit out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.percentage = bus.percentage();
  out.speed_limit = bus.speed_limit();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsSpeedLimitMapper
    : public TypedTopicMapper<Nav2MsgsSpeedLimitMapper, ::nav2_msgs::msg::SpeedLimit> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/SpeedLimit"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::SpeedLimit &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::speed_limit_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::SpeedLimit bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::SpeedLimit bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::speed_limit_to_ros(bus);
  }
};
#else
struct Nav2MsgsSpeedLimitMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/SpeedLimit"; }
};
#endif

}  // namespace robot_bus
