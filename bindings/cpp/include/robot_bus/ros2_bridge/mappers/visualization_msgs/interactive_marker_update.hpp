#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/interactive_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/interactive_marker.hpp>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/interactive_marker_pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/interactive_marker_update.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::InteractiveMarkerUpdate interactive_marker_update_to_bus(const ::visualization_msgs::msg::InteractiveMarkerUpdate &msg) {
  ::visualization_msgs::msg::v1::InteractiveMarkerUpdate bus;
  bus.set_server_id(msg.server_id.c_str());
  bus.set_seq_num(msg.seq_num);
  bus.set_type(msg.type);
  for (const auto &x : msg.markers) {
    *bus.add_markers() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_to_bus(x);
  }
  for (const auto &x : msg.poses) {
    *bus.add_poses() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_pose_to_bus(x);
  }
  for (const auto &x : msg.erases) {
    bus.add_erases(x.c_str());
  }
  return bus;
}

inline ::visualization_msgs::msg::InteractiveMarkerUpdate interactive_marker_update_to_ros(const ::visualization_msgs::msg::v1::InteractiveMarkerUpdate &bus) {
  ::visualization_msgs::msg::InteractiveMarkerUpdate out;
  out.server_id = bus.server_id();
  out.seq_num = bus.seq_num();
  out.type = bus.type();
  out.markers.clear();
  for (const auto &x : bus.markers()) {
    out.markers.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_to_ros(x));
  }
  out.poses.clear();
  for (const auto &x : bus.poses()) {
    out.poses.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_pose_to_ros(x));
  }
  out.erases.clear();
  for (const auto &x : bus.erases()) {
    out.erases.push_back(x);
  }
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsInteractiveMarkerUpdateMapper
    : public TypedTopicMapper<VisualizationMsgsInteractiveMarkerUpdateMapper, ::visualization_msgs::msg::InteractiveMarkerUpdate> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::InteractiveMarkerUpdate &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::interactive_marker_update_to_bus(msg);
    return encode_pb(bus);
  }

  ::visualization_msgs::msg::InteractiveMarkerUpdate bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::InteractiveMarkerUpdate bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::interactive_marker_update_to_ros(bus);
  }
};
#else
struct VisualizationMsgsInteractiveMarkerUpdateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
