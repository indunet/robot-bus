#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/quaternion.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/quaternion.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Quaternion quaternion_to_bus(const ::foxglove_msgs::msg::Quaternion &msg) {
  ::foxglove_msgs::msg::v1::Quaternion bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_z(msg.z);
  bus.set_w(msg.w);
  return bus;
}

inline ::foxglove_msgs::msg::Quaternion quaternion_to_ros(const ::foxglove_msgs::msg::v1::Quaternion &bus) {
  ::foxglove_msgs::msg::Quaternion out;
  out.x = bus.x();
  out.y = bus.y();
  out.z = bus.z();
  out.w = bus.w();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsQuaternionMapper
    : public TypedTopicMapper<FoxgloveMsgsQuaternionMapper, ::foxglove_msgs::msg::Quaternion> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/Quaternion"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Quaternion &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::quaternion_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Quaternion bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Quaternion bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::quaternion_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsQuaternionMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/Quaternion"; }
};
#endif

}  // namespace robot_bus
