#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/compressed_audio.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/compressed_audio.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::CompressedAudio compressed_audio_to_bus(const ::foxglove_msgs::msg::CompressedAudio &msg) {
  ::foxglove_msgs::msg::v1::CompressedAudio bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  bus.set_format(msg.format.c_str());
  return bus;
}

inline ::foxglove_msgs::msg::CompressedAudio compressed_audio_to_ros(const ::foxglove_msgs::msg::v1::CompressedAudio &bus) {
  ::foxglove_msgs::msg::CompressedAudio out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.data.assign(bus.data().begin(), bus.data().end());
  out.format = bus.format();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsCompressedAudioMapper
    : public TypedTopicMapper<FoxgloveMsgsCompressedAudioMapper, ::foxglove_msgs::msg::CompressedAudio> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::CompressedAudio &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::compressed_audio_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::CompressedAudio bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::CompressedAudio bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::compressed_audio_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsCompressedAudioMapper : TopicMapper {};
#endif

}  // namespace robot_bus
