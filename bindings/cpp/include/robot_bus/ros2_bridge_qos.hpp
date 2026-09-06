#pragma once

#include <robot_bus/node.hpp>

#include <cstdint>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

#ifdef ROBOT_BUS_HAS_ROS2
#include <atomic>
#include <chrono>
#include <mutex>
#include <rcl_action/action_client.h>
#include <rcl_action/action_server.h>
#include <rclcpp/rclcpp.hpp>
#include <rclcpp_action/rclcpp_action.hpp>
#endif

namespace robot_bus {

/// Topic / service / action bridge direction (ROS 2 ↔ robot-bus).
enum class Direction {
  Ros2ToBus = 0,
  BusToRos2 = 1,
};

/// True when this translation unit / consumer was built with `ROBOT_BUS_HAS_ROS2`
/// (link `robot_bus_ros2_bridge`). Independent of the C ABI stub.
inline bool ros2_available() {
#ifdef ROBOT_BUS_HAS_ROS2
  return true;
#else
  return false;
#endif
}

inline constexpr const char *kRos2BridgeUnavailable =
    "ROS 2 bridge not built: link robot_bus_ros2_bridge "
    "(CMake -DROBOT_BUS_ROS2=ON) after sourcing Humble/Jazzy";

/// Default timeout for bridged service calls (seconds).
inline constexpr double kServiceCallTimeoutSecs = 5.0;
/// Default timeout for bridged action goals (seconds).
inline constexpr double kActionCallTimeoutSecs = 30.0;

/// Intermediate after [`TopicQos::keep_last`]; finish with `.reliable()` or `.best_effort()`.
class TopicQos;

class TopicQosKeepLast {
 public:
  TopicQos reliable() const;
  TopicQos best_effort() const;

 private:
  friend class TopicQos;
  explicit TopicQosKeepLast(int32_t depth) : depth_(depth) {}
  int32_t depth_;
};

/// KeepLast depth plus reliability and optional ROS durability. Same type on
/// **ROS** and **bus** endpoints for topics, services, and actions. ROS honors
/// depth + reliability + durability. Bus uses depth as ZMQ HWM and must be
/// `.best_effort()` (no DDS reliability); durability is ignored on bus.
///
/// Usual routes use named presets (`sensor_data`, `ros_default`, `latched`,
/// `bus`). Custom depth still uses `keep_last(n).reliable()` / `.best_effort()`.
/// C++ uses `ros_default()` because `default` is a keyword (Python/Rust: `default()`).
class TopicQos {
 public:
  static TopicQosKeepLast keep_last(int32_t depth) { return TopicQosKeepLast(depth); }
  /// ROS `SensorDataQoS`: KeepLast 5, best effort, volatile.
  static TopicQos sensor_data() { return keep_last(5).best_effort(); }
  /// ROS `qos_profile_default` / `ServicesQoS`: KeepLast 10, reliable, volatile.
  /// Not ROS `SystemDefaultsQoS` (RMW/DDS vendor defaults).
  static TopicQos ros_default() { return keep_last(10).reliable(); }
  /// Latched ROS topics such as `/tf_static`: KeepLast 1, reliable, transient local.
  static TopicQos latched() { return keep_last(1).reliable().transient_local(); }
  /// Typical bus endpoint: KeepLast 8 (STREAM HWM), best effort.
  static TopicQos bus() { return keep_last(8).best_effort(); }
  int32_t depth() const { return depth_; }
  bool is_best_effort() const { return best_effort_; }
  bool is_reliable() const { return !best_effort_; }
  bool is_transient_local() const { return transient_local_; }
  bool is_volatile() const { return !transient_local_; }

  /// ROS `TRANSIENT_LOCAL` (latch). Needed for `/tf_static` and other latched topics.
  TopicQos transient_local() const { return TopicQos(depth_, best_effort_, true); }
  /// ROS `VOLATILE` (default). `volatile` is a C++ keyword, so this matches rclcpp.
  TopicQos durability_volatile() const { return TopicQos(depth_, best_effort_, false); }

 private:
  friend class TopicQosKeepLast;
  TopicQos(int32_t depth, bool best_effort, bool transient_local = false)
      : depth_(depth), best_effort_(best_effort), transient_local_(transient_local) {}
  int32_t depth_;
  bool best_effort_;
  bool transient_local_;
};

inline TopicQos TopicQosKeepLast::reliable() const { return TopicQos(depth_, false); }
inline TopicQos TopicQosKeepLast::best_effort() const { return TopicQos(depth_, true); }

inline void require_bus_best_effort(const TopicQos &qos) {
  if (qos.is_reliable()) {
    throw Error(
        "ros2 bridge: bus TopicQos must be .best_effort() "
        "(bus has no DDS reliability)");
  }
}

/// Point-in-time copy of per-bridge topic drop counters.
struct DropStatsSnapshot {
  uint64_t convert_fail = 0;
  uint64_t decode_fail = 0;
  uint64_t publish_fail = 0;
};

inline std::string qos_console_label(const TopicQos &qos) {
  std::string s = "keep_last(" + std::to_string(qos.depth()) + ").";
  s += qos.is_best_effort() ? "best_effort" : "reliable";
  if (qos.is_transient_local()) {
    s += ".transient_local";
  }
  return s;
}

inline const char *direction_label(Direction direction) {
  return direction == Direction::Ros2ToBus ? "ros→bus" : "bus→ros";
}

#ifdef ROBOT_BUS_HAS_ROS2
/// Atomic drop counters shared with ROS and bus callbacks.
struct DropStats {
  std::atomic<uint64_t> convert_fail{0};
  std::atomic<uint64_t> decode_fail{0};
  std::atomic<uint64_t> publish_fail{0};

  DropStatsSnapshot snapshot() const {
    return DropStatsSnapshot{
        convert_fail.load(std::memory_order_relaxed),
        decode_fail.load(std::memory_order_relaxed),
        publish_fail.load(std::memory_order_relaxed),
    };
  }
};

struct RouteHealth {
  std::atomic<uint64_t> rx{0};
  std::atomic<uint64_t> tx{0};
  std::atomic<uint64_t> convert_fail{0};
  std::atomic<uint64_t> decode_fail{0};
  std::atomic<uint64_t> publish_fail{0};
  std::atomic<uint64_t> last_rx_ms{0};
  std::atomic<uint64_t> last_warn_ms{0};
  std::atomic<bool> idle_latched{false};

  static uint64_t unix_ms() {
    return static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::milliseconds>(
                                     std::chrono::system_clock::now().time_since_epoch())
                                     .count());
  }

  void record_rx() {
    rx.fetch_add(1, std::memory_order_relaxed);
    last_rx_ms.store(unix_ms(), std::memory_order_relaxed);
  }
  void record_tx() { tx.fetch_add(1, std::memory_order_relaxed); }
  void record_convert_fail() { convert_fail.fetch_add(1, std::memory_order_relaxed); }
  void record_decode_fail() { decode_fail.fetch_add(1, std::memory_order_relaxed); }
  void record_publish_fail() { publish_fail.fetch_add(1, std::memory_order_relaxed); }

  bool should_log_warn() {
    const uint64_t now = unix_ms();
    const uint64_t prev = last_warn_ms.load(std::memory_order_relaxed);
    if (prev != 0 && now - prev < 1000) {
      return false;
    }
    last_warn_ms.store(now, std::memory_order_relaxed);
    return true;
  }

  bool is_idle(bool enabled, bool grace_elapsed) const {
    return enabled && grace_elapsed && last_rx_ms.load(std::memory_order_relaxed) == 0;
  }

  bool take_idle_event(bool enabled, bool grace_elapsed) {
    if (last_rx_ms.load(std::memory_order_relaxed) != 0) {
      idle_latched.store(false, std::memory_order_relaxed);
      return false;
    }
    if (!is_idle(enabled, grace_elapsed)) {
      return false;
    }
    return !idle_latched.exchange(true, std::memory_order_relaxed);
  }
};

class BridgeDecodeError : public Error {
 public:
  explicit BridgeDecodeError(std::string msg) : Error(std::move(msg)) {}
};

inline rclcpp::Logger ros2_bridge_logger() {
  return rclcpp::get_logger("robot_bus.ros2_bridge");
}

inline void note_convert_fail(const std::shared_ptr<DropStats> &stats, const char *dir,
                              const std::string &topic, const char *err,
                              const std::shared_ptr<RouteHealth> &health = {}) {
  if (stats) {
    stats->convert_fail.fetch_add(1, std::memory_order_relaxed);
  }
  if (health) {
    health->record_convert_fail();
  }
  if (!health || health->should_log_warn()) {
    RCLCPP_WARN(ros2_bridge_logger(), "%s %s convert: %s", dir, topic.c_str(), err);
  }
}

inline void note_decode_fail(const std::shared_ptr<DropStats> &stats, const char *dir,
                             const std::string &topic, const char *err,
                             const std::shared_ptr<RouteHealth> &health = {}) {
  if (stats) {
    stats->decode_fail.fetch_add(1, std::memory_order_relaxed);
  }
  if (health) {
    health->record_decode_fail();
  }
  if (!health || health->should_log_warn()) {
    RCLCPP_WARN(ros2_bridge_logger(), "%s %s decode: %s", dir, topic.c_str(), err);
  }
}

inline void note_publish_fail(const std::shared_ptr<DropStats> &stats, const char *dir,
                              const std::string &topic, const char *err,
                              const std::shared_ptr<RouteHealth> &health = {}) {
  if (stats) {
    stats->publish_fail.fetch_add(1, std::memory_order_relaxed);
  }
  if (health) {
    health->record_publish_fail();
  }
  if (!health || health->should_log_warn()) {
    RCLCPP_WARN(ros2_bridge_logger(), "%s %s publish: %s", dir, topic.c_str(), err);
  }
}

template <typename ConvertFn, typename PublishFn>
void forward_ros_to_bus(const std::shared_ptr<DropStats> &stats, const std::string &topic,
                        ConvertFn &&convert, PublishFn &&publish,
                        const std::shared_ptr<RouteHealth> &health = {}) {
  if (health) {
    health->record_rx();
  }
  std::vector<uint8_t> bytes;
  try {
    bytes = std::forward<ConvertFn>(convert)();
  } catch (const std::exception &e) {
    note_convert_fail(stats, "ros→bus", topic, e.what(), health);
    return;
  } catch (...) {
    note_convert_fail(stats, "ros→bus", topic, "unknown", health);
    return;
  }
  try {
    std::forward<PublishFn>(publish)(bytes);
    if (health) {
      health->record_tx();
    }
  } catch (const std::exception &e) {
    note_publish_fail(stats, "ros→bus", topic, e.what(), health);
  } catch (...) {
    note_publish_fail(stats, "ros→bus", topic, "unknown", health);
  }
}

template <typename ConvertFn, typename PublishFn>
void forward_bus_to_ros(const std::shared_ptr<DropStats> &stats, const std::string &topic,
                        ConvertFn &&convert, PublishFn &&publish,
                        const std::shared_ptr<RouteHealth> &health = {}) {
  if (health) {
    health->record_rx();
  }
  try {
    auto ros_msg = std::forward<ConvertFn>(convert)();
    try {
      std::forward<PublishFn>(publish)(std::move(ros_msg));
      if (health) {
        health->record_tx();
      }
    } catch (const std::exception &e) {
      note_publish_fail(stats, "bus→ros", topic, e.what(), health);
    } catch (...) {
      note_publish_fail(stats, "bus→ros", topic, "unknown", health);
    }
  } catch (const BridgeDecodeError &e) {
    note_decode_fail(stats, "bus→ros", topic, e.what(), health);
  } catch (const std::exception &e) {
    note_convert_fail(stats, "bus→ros", topic, e.what(), health);
  } catch (...) {
    note_convert_fail(stats, "bus→ros", topic, "unknown", health);
  }
}
#endif

#ifdef ROBOT_BUS_HAS_ROS2
inline rmw_qos_profile_t apply_keep_last_reliability(rmw_qos_profile_t base,
                                                     const TopicQos &qos) {
  base.history = RMW_QOS_POLICY_HISTORY_KEEP_LAST;
  base.depth = static_cast<size_t>(qos.depth() < 0 ? 0 : qos.depth());
  base.reliability = qos.is_best_effort() ? RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT
                                          : RMW_QOS_POLICY_RELIABILITY_RELIABLE;
  base.durability = qos.is_transient_local() ? RMW_QOS_POLICY_DURABILITY_TRANSIENT_LOCAL
                                             : RMW_QOS_POLICY_DURABILITY_VOLATILE;
  return base;
}

/// ROS service / action RPC QoS (`rmw_qos_profile_services_default` + KeepLast + reliability).
inline rmw_qos_profile_t service_rmw_qos(const TopicQos &qos) {
  return apply_keep_last_reliability(rmw_qos_profile_services_default, qos);
}

/// Action feedback topic QoS (`rmw_qos_profile_default` + KeepLast + reliability).
inline rmw_qos_profile_t action_feedback_rmw_qos(const TopicQos &qos) {
  return apply_keep_last_reliability(rmw_qos_profile_default, qos);
}

inline rcl_action_server_options_t action_server_qos(const TopicQos &qos) {
  auto opts = rcl_action_server_get_default_options();
  const auto srv = service_rmw_qos(qos);
  opts.goal_service_qos = srv;
  opts.result_service_qos = srv;
  opts.cancel_service_qos = srv;
  opts.feedback_topic_qos = action_feedback_rmw_qos(qos);
  return opts;
}

inline rcl_action_client_options_t action_client_qos(const TopicQos &qos) {
  auto opts = rcl_action_client_get_default_options();
  const auto srv = service_rmw_qos(qos);
  opts.goal_service_qos = srv;
  opts.result_service_qos = srv;
  opts.cancel_service_qos = srv;
  opts.feedback_topic_qos = action_feedback_rmw_qos(qos);
  return opts;
}

/// Context for custom [`TopicMapper::attach`].
struct TopicWireContext {
  rclcpp::Node::SharedPtr ros_node;
  Node &bus_node;
  const std::string &ros_topic;
  const std::string &bus_topic;
  Direction direction;
  rclcpp::QoS qos{10};
  /// Bus KeepLast HWM; `0` leaves the Node default.
  int32_t bus_qos_depth{0};
  /// Keep ROS / bus entities alive for the bridge lifetime.
  std::vector<std::shared_ptr<void>> &keep_alive;
  std::shared_ptr<DropStats> drop_stats;
  std::shared_ptr<RouteHealth> health;

  template <typename T>
  void retain(std::shared_ptr<T> p) {
    keep_alive.push_back(std::shared_ptr<void>(std::move(p)));
  }
};

/// Context for custom [`ServiceMapper::attach`].
struct ServiceWireContext {
  rclcpp::Node::SharedPtr ros_node;
  Node &bus_node;
  const std::string &ros_service;
  const std::string &bus_service;
  Direction direction;
  double timeout_secs;
  TopicQos ros_qos;
  TopicQos bus_qos;
  rclcpp::CallbackGroup::SharedPtr callback_group;
  std::vector<std::shared_ptr<void>> &keep_alive;

  template <typename T>
  void retain(std::shared_ptr<T> p) {
    keep_alive.push_back(std::shared_ptr<void>(std::move(p)));
  }
};

/// Context for custom [`ActionMapper::attach`].
struct ActionWireContext {
  rclcpp::Node::SharedPtr ros_node;
  Node &bus_node;
  const std::string &ros_action;
  const std::string &bus_action;
  Direction direction;
  double timeout_secs;
  TopicQos ros_qos;
  TopicQos bus_qos;
  rclcpp::CallbackGroup::SharedPtr callback_group;
  std::vector<std::shared_ptr<void>> &keep_alive;

  template <typename T>
  void retain(std::shared_ptr<T> p) {
    keep_alive.push_back(std::shared_ptr<void>(std::move(p)));
  }
};
#endif  // ROBOT_BUS_HAS_ROS2


}  // namespace robot_bus
