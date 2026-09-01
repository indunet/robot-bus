#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/point_cloud.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point32.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/channel_float32.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/point_cloud.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::PointCloud point_cloud_to_bus(const ::sensor_msgs::msg::PointCloud &msg) {
  ::sensor_msgs::msg::v1::PointCloud bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point32_to_bus(x);
  }
  for (const auto &x : msg.channels) {
    *bus.add_channels() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::channel_float32_to_bus(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::PointCloud point_cloud_to_ros(const ::sensor_msgs::msg::v1::PointCloud &bus) {
  ::sensor_msgs::msg::PointCloud out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::point32_to_ros(x));
  }
  out.channels.clear();
  for (const auto &x : bus.channels()) {
    out.channels.push_back(::robot_bus::ros2_bridge_mappers::sensor_msgs::channel_float32_to_ros(x));
  }
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsPointCloudMapper
    : public TypedTopicMapper<SensorMsgsPointCloudMapper, ::sensor_msgs::msg::PointCloud> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::PointCloud &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::point_cloud_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::PointCloud bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::PointCloud bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::point_cloud_to_ros(bus);
  }
};
#else
struct SensorMsgsPointCloudMapper : TopicMapper {};
#endif

}  // namespace robot_bus
