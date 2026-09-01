#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/point2.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/point2.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Point2 point2_to_bus(const ::foxglove_msgs::msg::Point2 &msg) {
  ::foxglove_msgs::msg::v1::Point2 bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  return bus;
}

inline ::foxglove_msgs::msg::Point2 point2_to_ros(const ::foxglove_msgs::msg::v1::Point2 &bus) {
  ::foxglove_msgs::msg::Point2 out;
  out.x = bus.x();
  out.y = bus.y();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPoint2Mapper
    : public TypedTopicMapper<FoxgloveMsgsPoint2Mapper, ::foxglove_msgs::msg::Point2> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/Point2"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Point2 &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::point2_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Point2 bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Point2 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::point2_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPoint2Mapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/Point2"; }
};
#endif

}  // namespace robot_bus
