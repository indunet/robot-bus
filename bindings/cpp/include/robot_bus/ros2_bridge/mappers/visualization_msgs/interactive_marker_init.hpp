#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/interactive_marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/interactive_marker.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/interactive_marker_init.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::InteractiveMarkerInit interactive_marker_init_to_bus(const ::visualization_msgs::msg::InteractiveMarkerInit &msg) {
  ::visualization_msgs::msg::v1::InteractiveMarkerInit bus;
  bus.set_server_id(msg.server_id.c_str());
  bus.set_seq_num(msg.seq_num);
  for (const auto &x : msg.markers) {
    *bus.add_markers() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_to_bus(x);
  }
  return bus;
}

inline ::visualization_msgs::msg::InteractiveMarkerInit interactive_marker_init_to_ros(const ::visualization_msgs::msg::v1::InteractiveMarkerInit &bus) {
  ::visualization_msgs::msg::InteractiveMarkerInit out;
  out.server_id = bus.server_id();
  out.seq_num = bus.seq_num();
  out.markers.clear();
  for (const auto &x : bus.markers()) {
    out.markers.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::interactive_marker_to_ros(x));
  }
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsInteractiveMarkerInitMapper
    : public TypedTopicMapper<VisualizationMsgsInteractiveMarkerInitMapper, ::visualization_msgs::msg::InteractiveMarkerInit> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::InteractiveMarkerInit &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::interactive_marker_init_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::InteractiveMarkerInit bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::InteractiveMarkerInit bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::interactive_marker_init_to_ros(bus);
  }
};
#else
struct VisualizationMsgsInteractiveMarkerInitMapper : TopicMapper {};
#endif

}  // namespace robot_bus
