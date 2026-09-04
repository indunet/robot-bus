#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/uv_coordinate.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/uv_coordinate.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::UVCoordinate uv_coordinate_to_bus(const ::visualization_msgs::msg::UVCoordinate &msg) {
  ::visualization_msgs::msg::v1::UVCoordinate bus;
  bus.set_u(msg.u);
  bus.set_v(msg.v);
  return bus;
}

inline ::visualization_msgs::msg::UVCoordinate uv_coordinate_to_ros(const ::visualization_msgs::msg::v1::UVCoordinate &bus) {
  ::visualization_msgs::msg::UVCoordinate out;
  out.u = bus.u();
  out.v = bus.v();
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsUvCoordinateMapper
    : public TypedTopicMapper<VisualizationMsgsUvCoordinateMapper, ::visualization_msgs::msg::UVCoordinate> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::UVCoordinate &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::uv_coordinate_to_bus(msg);
    return encode_pb(bus);
  }

  ::visualization_msgs::msg::UVCoordinate bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::UVCoordinate bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::uv_coordinate_to_ros(bus);
  }
};
#else
struct VisualizationMsgsUvCoordinateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
