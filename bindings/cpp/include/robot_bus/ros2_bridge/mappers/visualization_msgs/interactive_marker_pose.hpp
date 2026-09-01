#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/interactive_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/interactive_marker_pose.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::InteractiveMarkerPose interactive_marker_pose_to_bus(const ::visualization_msgs::msg::InteractiveMarkerPose &msg) {
  ::visualization_msgs::msg::v1::InteractiveMarkerPose bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  bus.set_name(msg.name.c_str());
  return bus;
}

inline ::visualization_msgs::msg::InteractiveMarkerPose interactive_marker_pose_to_ros(const ::visualization_msgs::msg::v1::InteractiveMarkerPose &bus) {
  ::visualization_msgs::msg::InteractiveMarkerPose out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  out.name = bus.name();
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsInteractiveMarkerPoseMapper
    : public TypedTopicMapper<VisualizationMsgsInteractiveMarkerPoseMapper, ::visualization_msgs::msg::InteractiveMarkerPose> {
 public:
  const char *type_name() const override { return "visualization_msgs/msg/InteractiveMarkerPose"; }

  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::InteractiveMarkerPose &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::interactive_marker_pose_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::InteractiveMarkerPose bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::InteractiveMarkerPose bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::interactive_marker_pose_to_ros(bus);
  }
};
#else
struct VisualizationMsgsInteractiveMarkerPoseMapper : TopicMapper {
  const char *type_name() const override { return "visualization_msgs/msg/InteractiveMarkerPose"; }
};
#endif

}  // namespace robot_bus
