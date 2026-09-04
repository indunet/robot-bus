#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/temperature.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/temperature.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::Temperature temperature_to_bus(const ::sensor_msgs::msg::Temperature &msg) {
  ::sensor_msgs::msg::v1::Temperature bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_temperature(msg.temperature);
  bus.set_variance(msg.variance);
  return bus;
}

inline ::sensor_msgs::msg::Temperature temperature_to_ros(const ::sensor_msgs::msg::v1::Temperature &bus) {
  ::sensor_msgs::msg::Temperature out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.temperature = bus.temperature();
  out.variance = bus.variance();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsTemperatureMapper
    : public TypedTopicMapper<SensorMsgsTemperatureMapper, ::sensor_msgs::msg::Temperature> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::Temperature &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::temperature_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::Temperature bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::Temperature bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::temperature_to_ros(bus);
  }
};
#else
struct SensorMsgsTemperatureMapper : TopicMapper {};
#endif

}  // namespace robot_bus
