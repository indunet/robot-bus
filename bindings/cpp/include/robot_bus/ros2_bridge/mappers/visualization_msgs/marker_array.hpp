#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/marker.pb.h>
#include <robot_bus/ros2_bridge/mappers/visualization_msgs/marker.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/marker_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::MarkerArray marker_array_to_bus(const ::visualization_msgs::msg::MarkerArray &msg) {
  ::visualization_msgs::msg::v1::MarkerArray bus;
  for (const auto &x : msg.markers) {
    *bus.add_markers() = ::robot_bus::ros2_bridge_mappers::visualization_msgs::marker_to_bus(x);
  }
  return bus;
}

inline ::visualization_msgs::msg::MarkerArray marker_array_to_ros(const ::visualization_msgs::msg::v1::MarkerArray &bus) {
  ::visualization_msgs::msg::MarkerArray out;
  out.markers.clear();
  for (const auto &x : bus.markers()) {
    out.markers.push_back(::robot_bus::ros2_bridge_mappers::visualization_msgs::marker_to_ros(x));
  }
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsMarkerArrayMapper
    : public TypedTopicMapper<VisualizationMsgsMarkerArrayMapper, ::visualization_msgs::msg::MarkerArray> {
 public:
  const char *type_name() const override { return "visualization_msgs/msg/MarkerArray"; }

  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::MarkerArray &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::marker_array_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::MarkerArray bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::MarkerArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::marker_array_to_ros(bus);
  }
};
#else
struct VisualizationMsgsMarkerArrayMapper : TopicMapper {
  const char *type_name() const override { return "visualization_msgs/msg/MarkerArray"; }
};
#endif

}  // namespace robot_bus
