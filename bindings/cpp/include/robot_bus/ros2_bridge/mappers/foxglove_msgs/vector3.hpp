#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/vector3.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/vector3.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Vector3 vector3_to_bus(const ::foxglove_msgs::msg::Vector3 &msg) {
  ::foxglove_msgs::msg::v1::Vector3 bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_z(msg.z);
  return bus;
}

inline ::foxglove_msgs::msg::Vector3 vector3_to_ros(const ::foxglove_msgs::msg::v1::Vector3 &bus) {
  ::foxglove_msgs::msg::Vector3 out;
  out.x = bus.x();
  out.y = bus.y();
  out.z = bus.z();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsVector3Mapper
    : public TypedTopicMapper<FoxgloveMsgsVector3Mapper, ::foxglove_msgs::msg::Vector3> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/Vector3"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Vector3 &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Vector3 bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Vector3 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsVector3Mapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/Vector3"; }
};
#endif

}  // namespace robot_bus
