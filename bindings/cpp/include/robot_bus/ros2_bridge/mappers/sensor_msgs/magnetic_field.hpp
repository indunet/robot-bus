#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/magnetic_field.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/magnetic_field.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::MagneticField magnetic_field_to_bus(const ::sensor_msgs::msg::MagneticField &msg) {
  ::sensor_msgs::msg::v1::MagneticField bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_magnetic_field() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.magnetic_field);
  for (auto x : msg.magnetic_field_covariance) {
    bus.add_magnetic_field_covariance(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::MagneticField magnetic_field_to_ros(const ::sensor_msgs::msg::v1::MagneticField &bus) {
  ::sensor_msgs::msg::MagneticField out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.magnetic_field = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.magnetic_field());
  ::robot_bus::ros2_bridge_mappers::copy_seq(out.magnetic_field_covariance, bus.magnetic_field_covariance());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsMagneticFieldMapper
    : public TypedTopicMapper<SensorMsgsMagneticFieldMapper, ::sensor_msgs::msg::MagneticField> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::MagneticField &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::magnetic_field_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::MagneticField bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::MagneticField bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::magnetic_field_to_ros(bus);
  }
};
#else
struct SensorMsgsMagneticFieldMapper : TopicMapper {};
#endif

}  // namespace robot_bus
