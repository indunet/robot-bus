#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/voxel_grid.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/packed_element_field.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/voxel_grid.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::VoxelGrid voxel_grid_to_bus(const ::foxglove_msgs::msg::VoxelGrid &msg) {
  ::foxglove_msgs::msg::v1::VoxelGrid bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_row_count(msg.row_count);
  bus.set_column_count(msg.column_count);
  *bus.mutable_cell_size() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.cell_size);
  bus.set_slice_stride(msg.slice_stride);
  bus.set_row_stride(msg.row_stride);
  bus.set_cell_stride(msg.cell_stride);
  for (const auto &x : msg.fields) {
    *bus.add_fields() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::packed_element_field_to_bus(x);
  }
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::foxglove_msgs::msg::VoxelGrid voxel_grid_to_ros(const ::foxglove_msgs::msg::v1::VoxelGrid &bus) {
  ::foxglove_msgs::msg::VoxelGrid out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.row_count = bus.row_count();
  out.column_count = bus.column_count();
  out.cell_size = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.cell_size());
  out.slice_stride = bus.slice_stride();
  out.row_stride = bus.row_stride();
  out.cell_stride = bus.cell_stride();
  out.fields.clear();
  for (const auto &x : bus.fields()) {
    out.fields.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::packed_element_field_to_ros(x));
  }
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsVoxelGridMapper
    : public TypedTopicMapper<FoxgloveMsgsVoxelGridMapper, ::foxglove_msgs::msg::VoxelGrid> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/VoxelGrid"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::VoxelGrid &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::voxel_grid_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::VoxelGrid bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::VoxelGrid bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::voxel_grid_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsVoxelGridMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/VoxelGrid"; }
};
#endif

}  // namespace robot_bus
