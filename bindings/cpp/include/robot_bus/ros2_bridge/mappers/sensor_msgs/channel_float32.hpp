#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/point_cloud.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/channel_float32.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::ChannelFloat32 channel_float32_to_bus(const ::sensor_msgs::msg::ChannelFloat32 &msg) {
  ::sensor_msgs::msg::v1::ChannelFloat32 bus;
  bus.set_name(msg.name.c_str());
  for (auto x : msg.values) {
    bus.add_values(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::ChannelFloat32 channel_float32_to_ros(const ::sensor_msgs::msg::v1::ChannelFloat32 &bus) {
  ::sensor_msgs::msg::ChannelFloat32 out;
  out.name = bus.name();
  out.values.assign(bus.values().begin(), bus.values().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsChannelFloat32Mapper
    : public TypedTopicMapper<SensorMsgsChannelFloat32Mapper, ::sensor_msgs::msg::ChannelFloat32> {
 public:
  const char *type_name() const override { return "sensor_msgs/msg/ChannelFloat32"; }

  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::ChannelFloat32 &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::channel_float32_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::ChannelFloat32 bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::ChannelFloat32 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::channel_float32_to_ros(bus);
  }
};
#else
struct SensorMsgsChannelFloat32Mapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/ChannelFloat32"; }
};
#endif

}  // namespace robot_bus
