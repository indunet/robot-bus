#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/illuminance.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/illuminance.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::Illuminance illuminance_to_bus(const ::sensor_msgs::msg::Illuminance &msg) {
  ::sensor_msgs::msg::v1::Illuminance bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_illuminance(msg.illuminance);
  bus.set_variance(msg.variance);
  return bus;
}

inline ::sensor_msgs::msg::Illuminance illuminance_to_ros(const ::sensor_msgs::msg::v1::Illuminance &bus) {
  ::sensor_msgs::msg::Illuminance out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.illuminance = bus.illuminance();
  out.variance = bus.variance();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsIlluminanceMapper
    : public TypedTopicMapper<SensorMsgsIlluminanceMapper, ::sensor_msgs::msg::Illuminance> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::Illuminance &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::illuminance_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::Illuminance bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::Illuminance bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::illuminance_to_ros(bus);
  }
};
#else
struct SensorMsgsIlluminanceMapper : TopicMapper {};
#endif

}  // namespace robot_bus
