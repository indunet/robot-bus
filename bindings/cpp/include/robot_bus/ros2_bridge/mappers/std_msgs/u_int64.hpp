#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/u_int64.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::UInt64 u_int64_to_bus(const ::std_msgs::msg::UInt64 &msg) {
  ::std_msgs::msg::v1::UInt64 bus;
  bus.set_data(msg.data);
  return bus;
}

inline ::std_msgs::msg::UInt64 u_int64_to_ros(const ::std_msgs::msg::v1::UInt64 &bus) {
  ::std_msgs::msg::UInt64 out;
  out.data = bus.data();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsUInt64Mapper
    : public TypedTopicMapper<StdMsgsUInt64Mapper, ::std_msgs::msg::UInt64> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::UInt64 &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::u_int64_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::UInt64 bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::UInt64 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::u_int64_to_ros(bus);
  }
};
#else
struct StdMsgsUInt64Mapper : TopicMapper {};
#endif

}  // namespace robot_bus
