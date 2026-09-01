#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/compressed_image.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/compressed_image.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::CompressedImage compressed_image_to_bus(const ::sensor_msgs::msg::CompressedImage &msg) {
  ::sensor_msgs::msg::v1::CompressedImage bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_format(msg.format.c_str());
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::sensor_msgs::msg::CompressedImage compressed_image_to_ros(const ::sensor_msgs::msg::v1::CompressedImage &bus) {
  ::sensor_msgs::msg::CompressedImage out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.format = bus.format();
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsCompressedImageMapper
    : public TypedTopicMapper<SensorMsgsCompressedImageMapper, ::sensor_msgs::msg::CompressedImage> {
 public:
  const char *type_name() const override { return "sensor_msgs/msg/CompressedImage"; }

  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::CompressedImage &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::compressed_image_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::CompressedImage bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::CompressedImage bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::compressed_image_to_ros(bus);
  }
};
#else
struct SensorMsgsCompressedImageMapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/CompressedImage"; }
};
#endif

}  // namespace robot_bus
