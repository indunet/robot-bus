#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/costmap.pb.h>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/costmap_meta_data.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::CostmapMetaData costmap_meta_data_to_bus(const ::nav2_msgs::msg::CostmapMetaData &msg) {
  ::nav2_msgs::msg::v1::CostmapMetaData bus;
  *bus.mutable_map_load_time() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.map_load_time);
  *bus.mutable_update_time() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.update_time);
  bus.set_resolution(msg.resolution);
  bus.set_size_x(msg.size_x);
  bus.set_size_y(msg.size_y);
  *bus.mutable_origin() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.origin);
  bus.set_layer(msg.layer.c_str());
  return bus;
}

inline ::nav2_msgs::msg::CostmapMetaData costmap_meta_data_to_ros(const ::nav2_msgs::msg::v1::CostmapMetaData &bus) {
  ::nav2_msgs::msg::CostmapMetaData out;
  out.map_load_time = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.map_load_time());
  out.update_time = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.update_time());
  out.resolution = bus.resolution();
  out.size_x = bus.size_x();
  out.size_y = bus.size_y();
  out.origin = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.origin());
  out.layer = bus.layer();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsCostmapMetaDataMapper
    : public TypedTopicMapper<Nav2MsgsCostmapMetaDataMapper, ::nav2_msgs::msg::CostmapMetaData> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::CostmapMetaData &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::costmap_meta_data_to_bus(msg);
    return encode_pb(bus);
  }

  ::nav2_msgs::msg::CostmapMetaData bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::CostmapMetaData bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::costmap_meta_data_to_ros(bus);
  }
};
#else
struct Nav2MsgsCostmapMetaDataMapper : TopicMapper {};
#endif

}  // namespace robot_bus
