#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/u_int8.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::UInt8 u_int8_to_bus(const ::std_msgs::msg::UInt8 &msg) {
  ::std_msgs::msg::v1::UInt8 bus;
  bus.set_data(static_cast<int32_t>(msg.data));
  return bus;
}

inline ::std_msgs::msg::UInt8 u_int8_to_ros(const ::std_msgs::msg::v1::UInt8 &bus) {
  ::std_msgs::msg::UInt8 out;
  out.data = static_cast<uint8_t>(bus.data());
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsUInt8Mapper
    : public TypedTopicMapper<StdMsgsUInt8Mapper, ::std_msgs::msg::UInt8> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::UInt8 &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::u_int8_to_bus(msg);
    return encode_pb(bus);
  }

  ::std_msgs::msg::UInt8 bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::UInt8 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::u_int8_to_ros(bus);
  }
};
#else
struct StdMsgsUInt8Mapper : TopicMapper {};
#endif

}  // namespace robot_bus
