#pragma once

#include <cstdint>
#include <string>
#include <vector>

#ifdef ROBOT_BUS_HAS_ROS2
#include <builtin_interfaces/msg/duration.hpp>
#include <builtin_interfaces/msg/time.hpp>
#include <google/protobuf/duration.pb.h>
#include <google/protobuf/timestamp.pb.h>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {

inline std::string i8_seq_to_bytes(const std::vector<int8_t> &data) {
  std::string out;
  out.resize(data.size());
  for (size_t i = 0; i < data.size(); ++i) {
    out[i] = static_cast<char>(static_cast<uint8_t>(data[i]));
  }
  return out;
}

inline std::vector<int8_t> bytes_to_i8_seq(const std::string &data) {
  std::vector<int8_t> out;
  out.reserve(data.size());
  for (unsigned char c : data) {
    out.push_back(static_cast<int8_t>(c));
  }
  return out;
}

#ifdef ROBOT_BUS_HAS_ROS2
inline google::protobuf::Timestamp time_to_timestamp(const builtin_interfaces::msg::Time &t) {
  google::protobuf::Timestamp out;
  out.set_seconds(t.sec);
  out.set_nanos(static_cast<int32_t>(t.nanosec));
  return out;
}

inline builtin_interfaces::msg::Time timestamp_to_time(const google::protobuf::Timestamp &t) {
  builtin_interfaces::msg::Time out;
  out.sec = static_cast<int32_t>(t.seconds());
  out.nanosec = static_cast<uint32_t>(t.nanos());
  return out;
}

inline google::protobuf::Duration duration_to_proto(const builtin_interfaces::msg::Duration &d) {
  google::protobuf::Duration out;
  out.set_seconds(d.sec);
  out.set_nanos(d.nanosec);
  return out;
}

inline builtin_interfaces::msg::Duration proto_to_duration(const google::protobuf::Duration &d) {
  builtin_interfaces::msg::Duration out;
  out.sec = static_cast<int32_t>(d.seconds());
  out.nanosec = d.nanos();
  return out;
}
#endif

}  // namespace ros2_bridge_mappers
}  // namespace robot_bus
