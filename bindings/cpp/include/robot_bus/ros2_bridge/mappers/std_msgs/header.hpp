#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/header.pb.h>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/header.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::Header header_to_bus(const ::std_msgs::msg::Header &msg) {
  ::std_msgs::msg::v1::Header bus;
  *bus.mutable_stamp() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.stamp);
  bus.set_frame_id(msg.frame_id.c_str());
  return bus;
}

inline ::std_msgs::msg::Header header_to_ros(const ::std_msgs::msg::v1::Header &bus) {
  ::std_msgs::msg::Header out;
  out.stamp = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.stamp());
  out.frame_id = bus.frame_id();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsHeaderMapper
    : public TypedTopicMapper<StdMsgsHeaderMapper, ::std_msgs::msg::Header> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::Header &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::header_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::Header bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::Header bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::header_to_ros(bus);
  }
};
#else
struct StdMsgsHeaderMapper : TopicMapper {};
#endif

}  // namespace robot_bus
