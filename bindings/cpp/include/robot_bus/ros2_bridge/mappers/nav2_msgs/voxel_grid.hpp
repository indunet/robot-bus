#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/voxel_grid.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point32.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/voxel_grid.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::VoxelGrid voxel_grid_to_bus(const ::nav2_msgs::msg::VoxelGrid &msg) {
  ::nav2_msgs::msg::v1::VoxelGrid bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (auto x : msg.data) {
    bus.add_data(x);
  }
  *bus.mutable_origin() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point32_to_bus(msg.origin);
  *bus.mutable_resolutions() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.resolutions);
  bus.set_size_x(msg.size_x);
  bus.set_size_y(msg.size_y);
  bus.set_size_z(msg.size_z);
  return bus;
}

inline ::nav2_msgs::msg::VoxelGrid voxel_grid_to_ros(const ::nav2_msgs::msg::v1::VoxelGrid &bus) {
  ::nav2_msgs::msg::VoxelGrid out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.data.assign(bus.data().begin(), bus.data().end());
  out.origin = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point32_to_ros(bus.origin());
  out.resolutions = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.resolutions());
  out.size_x = bus.size_x();
  out.size_y = bus.size_y();
  out.size_z = bus.size_z();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsVoxelGridMapper
    : public TypedTopicMapper<Nav2MsgsVoxelGridMapper, ::nav2_msgs::msg::VoxelGrid> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::VoxelGrid &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::voxel_grid_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::VoxelGrid bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::VoxelGrid bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::voxel_grid_to_ros(bus);
  }
};
#else
struct Nav2MsgsVoxelGridMapper : TopicMapper {};
#endif

}  // namespace robot_bus
