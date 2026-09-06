#pragma once

#include <algorithm>
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

/// `std::vector` (and ROS bounded vectors) have `assign`; ROS fixed arrays do not.
template <typename Dst, typename Src>
auto copy_seq(Dst &dst, const Src &src, int)
    -> decltype(dst.assign(src.begin(), src.end()), void()) {
  dst.assign(src.begin(), src.end());
}

template <typename Dst, typename Src>
void copy_seq(Dst &dst, const Src &src, long) {
  using size_type = decltype(dst.size());
  const auto n = std::min(dst.size(), static_cast<size_type>(src.size()));
  std::copy_n(src.begin(), n, dst.begin());
}

template <typename Dst, typename Src>
void copy_seq(Dst &dst, const Src &src) {
  copy_seq(dst, src, 0);
}

inline std::string i8_seq_to_bytes(const std::vector<int8_t> &data) {
  if (data.empty()) {
    return {};
  }
  return std::string(reinterpret_cast<const char *>(data.data()), data.size());
}

inline std::vector<int8_t> bytes_to_i8_seq(const std::string &data) {
  if (data.empty()) {
    return {};
  }
  const auto *p = reinterpret_cast<const int8_t *>(data.data());
  return std::vector<int8_t>(p, p + data.size());
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
