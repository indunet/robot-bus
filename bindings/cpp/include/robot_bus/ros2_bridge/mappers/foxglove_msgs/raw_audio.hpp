#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/raw_audio.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/raw_audio.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::RawAudio raw_audio_to_bus(const ::foxglove_msgs::msg::RawAudio &msg) {
  ::foxglove_msgs::msg::v1::RawAudio bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  bus.set_format(msg.format.c_str());
  bus.set_sample_rate(msg.sample_rate);
  bus.set_number_of_channels(msg.number_of_channels);
  return bus;
}

inline ::foxglove_msgs::msg::RawAudio raw_audio_to_ros(const ::foxglove_msgs::msg::v1::RawAudio &bus) {
  ::foxglove_msgs::msg::RawAudio out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.data.assign(bus.data().begin(), bus.data().end());
  out.format = bus.format();
  out.sample_rate = bus.sample_rate();
  out.number_of_channels = bus.number_of_channels();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsRawAudioMapper
    : public TypedTopicMapper<FoxgloveMsgsRawAudioMapper, ::foxglove_msgs::msg::RawAudio> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/RawAudio"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::RawAudio &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::raw_audio_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::RawAudio bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::RawAudio bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::raw_audio_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsRawAudioMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/RawAudio"; }
};
#endif

}  // namespace robot_bus
