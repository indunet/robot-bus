#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/occupancy_grid.pb.h>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/map_meta_data.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::MapMetaData map_meta_data_to_bus(const ::nav_msgs::msg::MapMetaData &msg) {
  ::nav_msgs::msg::v1::MapMetaData bus;
  *bus.mutable_map_load_time() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.map_load_time);
  bus.set_resolution(msg.resolution);
  bus.set_width(msg.width);
  bus.set_height(msg.height);
  *bus.mutable_origin() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.origin);
  return bus;
}

inline ::nav_msgs::msg::MapMetaData map_meta_data_to_ros(const ::nav_msgs::msg::v1::MapMetaData &bus) {
  ::nav_msgs::msg::MapMetaData out;
  out.map_load_time = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.map_load_time());
  out.resolution = bus.resolution();
  out.width = bus.width();
  out.height = bus.height();
  out.origin = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.origin());
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsMapMetaDataMapper
    : public TypedTopicMapper<NavMsgsMapMetaDataMapper, ::nav_msgs::msg::MapMetaData> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::MapMetaData &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::map_meta_data_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav_msgs::msg::MapMetaData bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::MapMetaData bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::map_meta_data_to_ros(bus);
  }
};
#else
struct NavMsgsMapMetaDataMapper : TopicMapper {};
#endif

}  // namespace robot_bus
