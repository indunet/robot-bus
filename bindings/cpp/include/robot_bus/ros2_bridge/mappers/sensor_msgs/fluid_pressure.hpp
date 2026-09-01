#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/fluid_pressure.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/fluid_pressure.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::FluidPressure fluid_pressure_to_bus(const ::sensor_msgs::msg::FluidPressure &msg) {
  ::sensor_msgs::msg::v1::FluidPressure bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_fluid_pressure(msg.fluid_pressure);
  bus.set_variance(msg.variance);
  return bus;
}

inline ::sensor_msgs::msg::FluidPressure fluid_pressure_to_ros(const ::sensor_msgs::msg::v1::FluidPressure &bus) {
  ::sensor_msgs::msg::FluidPressure out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.fluid_pressure = bus.fluid_pressure();
  out.variance = bus.variance();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsFluidPressureMapper
    : public TypedTopicMapper<SensorMsgsFluidPressureMapper, ::sensor_msgs::msg::FluidPressure> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::FluidPressure &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::fluid_pressure_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::FluidPressure bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::FluidPressure bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::fluid_pressure_to_ros(bus);
  }
};
#else
struct SensorMsgsFluidPressureMapper : TopicMapper {};
#endif

}  // namespace robot_bus
