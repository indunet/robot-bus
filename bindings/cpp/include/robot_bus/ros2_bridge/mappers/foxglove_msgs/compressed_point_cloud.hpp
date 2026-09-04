#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/compressed_point_cloud.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/compressed_point_cloud.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::CompressedPointCloud compressed_point_cloud_to_bus(const ::foxglove_msgs::msg::CompressedPointCloud &msg) {
  ::foxglove_msgs::msg::v1::CompressedPointCloud bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  bus.set_format(msg.format.c_str());
  return bus;
}

inline ::foxglove_msgs::msg::CompressedPointCloud compressed_point_cloud_to_ros(const ::foxglove_msgs::msg::v1::CompressedPointCloud &bus) {
  ::foxglove_msgs::msg::CompressedPointCloud out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.data.assign(bus.data().begin(), bus.data().end());
  out.format = bus.format();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsCompressedPointCloudMapper
    : public TypedTopicMapper<FoxgloveMsgsCompressedPointCloudMapper, ::foxglove_msgs::msg::CompressedPointCloud> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::CompressedPointCloud &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::compressed_point_cloud_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::CompressedPointCloud bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::CompressedPointCloud bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::compressed_point_cloud_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsCompressedPointCloudMapper : TopicMapper {};
#endif

}  // namespace robot_bus
