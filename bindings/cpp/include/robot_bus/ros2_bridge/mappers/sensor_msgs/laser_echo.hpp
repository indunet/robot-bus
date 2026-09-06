#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/multi_echo_laser_scan.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/laser_echo.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::LaserEcho laser_echo_to_bus(const ::sensor_msgs::msg::LaserEcho &msg) {
  ::sensor_msgs::msg::v1::LaserEcho bus;
  for (auto x : msg.echoes) {
    bus.add_echoes(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::LaserEcho laser_echo_to_ros(const ::sensor_msgs::msg::v1::LaserEcho &bus) {
  ::sensor_msgs::msg::LaserEcho out;
  ::robot_bus::ros2_bridge_mappers::copy_seq(out.echoes, bus.echoes());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsLaserEchoMapper
    : public TypedTopicMapper<SensorMsgsLaserEchoMapper, ::sensor_msgs::msg::LaserEcho> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::LaserEcho &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::laser_echo_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::LaserEcho bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::LaserEcho bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::laser_echo_to_ros(bus);
  }
};
#else
struct SensorMsgsLaserEchoMapper : TopicMapper {};
#endif

}  // namespace robot_bus
