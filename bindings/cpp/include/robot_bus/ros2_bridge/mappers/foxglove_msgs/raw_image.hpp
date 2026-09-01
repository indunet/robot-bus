#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/raw_image.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/raw_image.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::RawImage raw_image_to_bus(const ::foxglove_msgs::msg::RawImage &msg) {
  ::foxglove_msgs::msg::v1::RawImage bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  bus.set_width(msg.width);
  bus.set_height(msg.height);
  bus.set_encoding(msg.encoding.c_str());
  bus.set_step(msg.step);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::foxglove_msgs::msg::RawImage raw_image_to_ros(const ::foxglove_msgs::msg::v1::RawImage &bus) {
  ::foxglove_msgs::msg::RawImage out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.width = bus.width();
  out.height = bus.height();
  out.encoding = bus.encoding();
  out.step = bus.step();
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsRawImageMapper
    : public TypedTopicMapper<FoxgloveMsgsRawImageMapper, ::foxglove_msgs::msg::RawImage> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::RawImage &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::raw_image_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::RawImage bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::RawImage bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::raw_image_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsRawImageMapper : TopicMapper {};
#endif

}  // namespace robot_bus
