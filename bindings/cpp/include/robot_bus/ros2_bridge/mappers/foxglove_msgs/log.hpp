#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/log.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/log.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Log log_to_bus(const ::foxglove_msgs::msg::Log &msg) {
  ::foxglove_msgs::msg::v1::Log bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_level(static_cast<int32_t>(msg.level));
  bus.set_message(msg.message.c_str());
  bus.set_name(msg.name.c_str());
  bus.set_file(msg.file.c_str());
  bus.set_line(msg.line);
  return bus;
}

inline ::foxglove_msgs::msg::Log log_to_ros(const ::foxglove_msgs::msg::v1::Log &bus) {
  ::foxglove_msgs::msg::Log out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.level = bus.level();
  out.message = bus.message();
  out.name = bus.name();
  out.file = bus.file();
  out.line = bus.line();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsLogMapper
    : public TypedTopicMapper<FoxgloveMsgsLogMapper, ::foxglove_msgs::msg::Log> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Log &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::log_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Log bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Log bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::log_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsLogMapper : TopicMapper {};
#endif

}  // namespace robot_bus
