#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/pose_in_frame.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/pose_in_frame.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::PoseInFrame pose_in_frame_to_bus(const ::foxglove_msgs::msg::PoseInFrame &msg) {
  ::foxglove_msgs::msg::v1::PoseInFrame bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  return bus;
}

inline ::foxglove_msgs::msg::PoseInFrame pose_in_frame_to_ros(const ::foxglove_msgs::msg::v1::PoseInFrame &bus) {
  ::foxglove_msgs::msg::PoseInFrame out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPoseInFrameMapper
    : public TypedTopicMapper<FoxgloveMsgsPoseInFrameMapper, ::foxglove_msgs::msg::PoseInFrame> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::PoseInFrame &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::pose_in_frame_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::PoseInFrame bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::PoseInFrame bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::pose_in_frame_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPoseInFrameMapper : TopicMapper {};
#endif

}  // namespace robot_bus
