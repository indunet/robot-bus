#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/empty.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/empty.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::Empty empty_to_bus(const ::std_msgs::msg::Empty &msg) {
  ::std_msgs::msg::v1::Empty bus;

  return bus;
}

inline ::std_msgs::msg::Empty empty_to_ros(const ::std_msgs::msg::v1::Empty &bus) {
  ::std_msgs::msg::Empty out;

  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsEmptyMapper
    : public TypedTopicMapper<StdMsgsEmptyMapper, ::std_msgs::msg::Empty> {
 public:
  const char *type_name() const override { return "std_msgs/msg/Empty"; }

  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::Empty &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::empty_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::Empty bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::Empty bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::empty_to_ros(bus);
  }
};
#else
struct StdMsgsEmptyMapper : TopicMapper {
  const char *type_name() const override { return "std_msgs/msg/Empty"; }
};
#endif

}  // namespace robot_bus
