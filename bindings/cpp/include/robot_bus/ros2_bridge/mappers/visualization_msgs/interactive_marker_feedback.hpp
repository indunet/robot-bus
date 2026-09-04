#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/interactive_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/interactive_marker_feedback.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::InteractiveMarkerFeedback interactive_marker_feedback_to_bus(const ::visualization_msgs::msg::InteractiveMarkerFeedback &msg) {
  ::visualization_msgs::msg::v1::InteractiveMarkerFeedback bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_client_id(msg.client_id.c_str());
  bus.set_marker_name(msg.marker_name.c_str());
  bus.set_control_name(msg.control_name.c_str());
  bus.set_event_type(msg.event_type);
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  bus.set_menu_entry_id(msg.menu_entry_id);
  *bus.mutable_mouse_point() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(msg.mouse_point);
  bus.set_mouse_point_valid(msg.mouse_point_valid);
  return bus;
}

inline ::visualization_msgs::msg::InteractiveMarkerFeedback interactive_marker_feedback_to_ros(const ::visualization_msgs::msg::v1::InteractiveMarkerFeedback &bus) {
  ::visualization_msgs::msg::InteractiveMarkerFeedback out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.client_id = bus.client_id();
  out.marker_name = bus.marker_name();
  out.control_name = bus.control_name();
  out.event_type = bus.event_type();
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  out.menu_entry_id = bus.menu_entry_id();
  out.mouse_point = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(bus.mouse_point());
  out.mouse_point_valid = bus.mouse_point_valid();
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsInteractiveMarkerFeedbackMapper
    : public TypedTopicMapper<VisualizationMsgsInteractiveMarkerFeedbackMapper, ::visualization_msgs::msg::InteractiveMarkerFeedback> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::InteractiveMarkerFeedback &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::interactive_marker_feedback_to_bus(msg);
    return encode_pb(bus);
  }

  ::visualization_msgs::msg::InteractiveMarkerFeedback bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::InteractiveMarkerFeedback bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::interactive_marker_feedback_to_ros(bus);
  }
};
#else
struct VisualizationMsgsInteractiveMarkerFeedbackMapper : TopicMapper {};
#endif

}  // namespace robot_bus
