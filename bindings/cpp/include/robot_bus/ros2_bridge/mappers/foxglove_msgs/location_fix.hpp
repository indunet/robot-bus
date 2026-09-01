#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/location_fix.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/location_fix.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::LocationFix location_fix_to_bus(const ::foxglove_msgs::msg::LocationFix &msg) {
  ::foxglove_msgs::msg::v1::LocationFix bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  bus.set_latitude(msg.latitude);
  bus.set_longitude(msg.longitude);
  bus.set_altitude(msg.altitude);
  for (auto x : msg.position_covariance) {
    bus.add_position_covariance(x);
  }
  bus.set_position_covariance_type(static_cast<int32_t>(msg.position_covariance_type));
  bus.set_heading(msg.heading);
  *bus.mutable_velocity() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.velocity);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::LocationFix location_fix_to_ros(const ::foxglove_msgs::msg::v1::LocationFix &bus) {
  ::foxglove_msgs::msg::LocationFix out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.latitude = bus.latitude();
  out.longitude = bus.longitude();
  out.altitude = bus.altitude();
  out.position_covariance.assign(bus.position_covariance().begin(), bus.position_covariance().end());
  out.position_covariance_type = bus.position_covariance_type();
  out.heading = bus.heading();
  out.velocity = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.velocity());
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  out.metadata.clear();
  for (const auto &x : bus.metadata()) {
    out.metadata.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsLocationFixMapper
    : public TypedTopicMapper<FoxgloveMsgsLocationFixMapper, ::foxglove_msgs::msg::LocationFix> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::LocationFix &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::location_fix_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::LocationFix bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::LocationFix bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::location_fix_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsLocationFixMapper : TopicMapper {};
#endif

}  // namespace robot_bus
