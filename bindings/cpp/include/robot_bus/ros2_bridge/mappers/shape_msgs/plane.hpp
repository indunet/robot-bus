#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/shape_msgs/msg/v1/plane.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <shape_msgs/msg/plane.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace shape_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::shape_msgs::msg::v1::Plane plane_to_bus(const ::shape_msgs::msg::Plane &msg) {
  ::shape_msgs::msg::v1::Plane bus;
  for (auto x : msg.coef) {
    bus.add_coef(x);
  }
  return bus;
}

inline ::shape_msgs::msg::Plane plane_to_ros(const ::shape_msgs::msg::v1::Plane &bus) {
  ::shape_msgs::msg::Plane out;
  out.coef.assign(bus.coef().begin(), bus.coef().end());
  return out;
}
#endif

}  // namespace shape_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ShapeMsgsPlaneMapper
    : public TypedTopicMapper<ShapeMsgsPlaneMapper, ::shape_msgs::msg::Plane> {
 public:
  const char *type_name() const override { return "shape_msgs/msg/Plane"; }

  std::vector<uint8_t> ros_to_bus(const ::shape_msgs::msg::Plane &msg) const {
    auto bus = ros2_bridge_mappers::shape_msgs::plane_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::shape_msgs::msg::Plane bus_to_ros(BytesView payload) const {
    ::shape_msgs::msg::v1::Plane bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::shape_msgs::plane_to_ros(bus);
  }
};
#else
struct ShapeMsgsPlaneMapper : TopicMapper {
  const char *type_name() const override { return "shape_msgs/msg/Plane"; }
};
#endif

}  // namespace robot_bus
