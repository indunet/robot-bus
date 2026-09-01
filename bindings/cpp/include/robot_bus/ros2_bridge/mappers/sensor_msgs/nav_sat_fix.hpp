#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/nav_sat_fix.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/nav_sat_status.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/nav_sat_fix.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::NavSatFix nav_sat_fix_to_bus(const ::sensor_msgs::msg::NavSatFix &msg) {
  ::sensor_msgs::msg::v1::NavSatFix bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_status() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::nav_sat_status_to_bus(msg.status);
  bus.set_latitude(msg.latitude);
  bus.set_longitude(msg.longitude);
  bus.set_altitude(msg.altitude);
  for (auto x : msg.position_covariance) {
    bus.add_position_covariance(x);
  }
  bus.set_position_covariance_type(msg.position_covariance_type);
  return bus;
}

inline ::sensor_msgs::msg::NavSatFix nav_sat_fix_to_ros(const ::sensor_msgs::msg::v1::NavSatFix &bus) {
  ::sensor_msgs::msg::NavSatFix out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.status = ::robot_bus::ros2_bridge_mappers::sensor_msgs::nav_sat_status_to_ros(bus.status());
  out.latitude = bus.latitude();
  out.longitude = bus.longitude();
  out.altitude = bus.altitude();
  out.position_covariance.assign(bus.position_covariance().begin(), bus.position_covariance().end());
  out.position_covariance_type = bus.position_covariance_type();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsNavSatFixMapper
    : public TypedTopicMapper<SensorMsgsNavSatFixMapper, ::sensor_msgs::msg::NavSatFix> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::NavSatFix &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::nav_sat_fix_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::NavSatFix bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::NavSatFix bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::nav_sat_fix_to_ros(bus);
  }
};
#else
struct SensorMsgsNavSatFixMapper : TopicMapper {};
#endif

}  // namespace robot_bus
