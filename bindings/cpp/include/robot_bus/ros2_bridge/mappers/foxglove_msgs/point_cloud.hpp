#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/point_cloud.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/packed_element_field.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/point_cloud.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::PointCloud point_cloud_to_bus(const ::foxglove_msgs::msg::PointCloud &msg) {
  ::foxglove_msgs::msg::v1::PointCloud bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_point_stride(msg.point_stride);
  for (const auto &x : msg.fields) {
    *bus.add_fields() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::packed_element_field_to_bus(x);
  }
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::foxglove_msgs::msg::PointCloud point_cloud_to_ros(const ::foxglove_msgs::msg::v1::PointCloud &bus) {
  ::foxglove_msgs::msg::PointCloud out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.point_stride = bus.point_stride();
  out.fields.clear();
  for (const auto &x : bus.fields()) {
    out.fields.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::packed_element_field_to_ros(x));
  }
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPointCloudMapper
    : public TypedTopicMapper<FoxgloveMsgsPointCloudMapper, ::foxglove_msgs::msg::PointCloud> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::PointCloud &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::point_cloud_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::PointCloud bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::PointCloud bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::point_cloud_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPointCloudMapper : TopicMapper {};
#endif

}  // namespace robot_bus
