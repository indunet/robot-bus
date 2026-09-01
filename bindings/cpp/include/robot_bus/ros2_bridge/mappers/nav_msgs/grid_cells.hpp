#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/grid_cells.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/grid_cells.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::GridCells grid_cells_to_bus(const ::nav_msgs::msg::GridCells &msg) {
  ::nav_msgs::msg::v1::GridCells bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_cell_width(msg.cell_width);
  bus.set_cell_height(msg.cell_height);
  for (const auto &x : msg.cells) {
    *bus.add_cells() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(x);
  }
  return bus;
}

inline ::nav_msgs::msg::GridCells grid_cells_to_ros(const ::nav_msgs::msg::v1::GridCells &bus) {
  ::nav_msgs::msg::GridCells out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.cell_width = bus.cell_width();
  out.cell_height = bus.cell_height();
  out.cells.clear();
  for (const auto &x : bus.cells()) {
    out.cells.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsGridCellsMapper
    : public TypedTopicMapper<NavMsgsGridCellsMapper, ::nav_msgs::msg::GridCells> {
 public:
  const char *type_name() const override { return "nav_msgs/msg/GridCells"; }

  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::GridCells &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::grid_cells_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav_msgs::msg::GridCells bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::GridCells bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::grid_cells_to_ros(bus);
  }
};
#else
struct NavMsgsGridCellsMapper : TopicMapper {
  const char *type_name() const override { return "nav_msgs/msg/GridCells"; }
};
#endif

}  // namespace robot_bus
