#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/occupancy_grid.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/nav_msgs/map_meta_data.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/occupancy_grid.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::OccupancyGrid occupancy_grid_to_bus(const ::nav_msgs::msg::OccupancyGrid &msg) {
  ::nav_msgs::msg::v1::OccupancyGrid bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_info() = ::robot_bus::ros2_bridge_mappers::nav_msgs::map_meta_data_to_bus(msg.info);
  {
    auto tmp = ::robot_bus::ros2_bridge_mappers::i8_seq_to_bytes(msg.data);
    bus.set_data(tmp.data(), tmp.size());
  }
  return bus;
}

inline ::nav_msgs::msg::OccupancyGrid occupancy_grid_to_ros(const ::nav_msgs::msg::v1::OccupancyGrid &bus) {
  ::nav_msgs::msg::OccupancyGrid out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.info = ::robot_bus::ros2_bridge_mappers::nav_msgs::map_meta_data_to_ros(bus.info());
  out.data = ::robot_bus::ros2_bridge_mappers::bytes_to_i8_seq(bus.data());
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsOccupancyGridMapper
    : public TypedTopicMapper<NavMsgsOccupancyGridMapper, ::nav_msgs::msg::OccupancyGrid> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::OccupancyGrid &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::occupancy_grid_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav_msgs::msg::OccupancyGrid bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::OccupancyGrid bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::occupancy_grid_to_ros(bus);
  }
};
#else
struct NavMsgsOccupancyGridMapper : TopicMapper {};
#endif

}  // namespace robot_bus
