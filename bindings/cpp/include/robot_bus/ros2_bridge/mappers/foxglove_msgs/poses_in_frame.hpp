#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/poses_in_frame.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/poses_in_frame.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::PosesInFrame poses_in_frame_to_bus(const ::foxglove_msgs::msg::PosesInFrame &msg) {
  ::foxglove_msgs::msg::v1::PosesInFrame bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  for (const auto &x : msg.poses) {
    *bus.add_poses() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::PosesInFrame poses_in_frame_to_ros(const ::foxglove_msgs::msg::v1::PosesInFrame &bus) {
  ::foxglove_msgs::msg::PosesInFrame out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.poses.clear();
  for (const auto &x : bus.poses()) {
    out.poses.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPosesInFrameMapper
    : public TypedTopicMapper<FoxgloveMsgsPosesInFrameMapper, ::foxglove_msgs::msg::PosesInFrame> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::PosesInFrame &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::poses_in_frame_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::PosesInFrame bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::PosesInFrame bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::poses_in_frame_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPosesInFrameMapper : TopicMapper {};
#endif

}  // namespace robot_bus
