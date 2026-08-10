#pragma once

#include <robot_bus.h>

#include <robot_bus/Node.hpp>

#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <utility>
#include <vector>

namespace robot_bus {

/// Topic / service / action bridge direction (ROS 2 ↔ robot-bus).
enum class Direction {
  Ros2ToBus = ROBOT_BUS_ROUTE_DIR_ROS2_TO_BUS,
  BusToRos2 = ROBOT_BUS_ROUTE_DIR_BUS_TO_ROS2,
};

/// True if `librobot_bus` was built with `--features ros2`.
inline bool ros2_available() { return robot_bus_ros2_available() != 0; }

/// Borrowed DynamicMessage view during a TopicMapper callback (fields via dotted paths).
class DynMsg {
 public:
  explicit DynMsg(RobotBusRos2DynMsg *raw) : raw_(raw) {}
  explicit DynMsg(const RobotBusRos2DynMsg *raw) : raw_(const_cast<RobotBusRos2DynMsg *>(raw)) {}

  RobotBusRos2DynMsg *raw() const { return raw_; }

  bool has_field(const char *path) const {
    return robot_bus_ros2_dyn_msg_has_field(raw_, path) != 0;
  }

  std::string get_string(const char *path) const {
    char *out = nullptr;
    check(robot_bus_ros2_dyn_msg_get_string(raw_, path, &out), "DynMsg::get_string");
    std::string s = out ? out : "";
    robot_bus_free_string(out);
    return s;
  }

  void set_string(const char *path, const std::string &value) {
    check(robot_bus_ros2_dyn_msg_set_string(raw_, path, value.c_str()), "DynMsg::set_string");
  }

  bool get_bool(const char *path) const {
    int out = 0;
    check(robot_bus_ros2_dyn_msg_get_bool(raw_, path, &out), "DynMsg::get_bool");
    return out != 0;
  }

  void set_bool(const char *path, bool value) {
    check(robot_bus_ros2_dyn_msg_set_bool(raw_, path, value ? 1 : 0), "DynMsg::set_bool");
  }

  int64_t get_i64(const char *path) const {
    int64_t out = 0;
    check(robot_bus_ros2_dyn_msg_get_i64(raw_, path, &out), "DynMsg::get_i64");
    return out;
  }

  void set_i64(const char *path, int64_t value) {
    check(robot_bus_ros2_dyn_msg_set_i64(raw_, path, value), "DynMsg::set_i64");
  }

  double get_f64(const char *path) const {
    double out = 0;
    check(robot_bus_ros2_dyn_msg_get_f64(raw_, path, &out), "DynMsg::get_f64");
    return out;
  }

  void set_f64(const char *path, double value) {
    check(robot_bus_ros2_dyn_msg_set_f64(raw_, path, value), "DynMsg::set_f64");
  }

  std::vector<uint8_t> get_bytes(const char *path) const {
    uint8_t *data = nullptr;
    size_t len = 0;
    check(robot_bus_ros2_dyn_msg_get_bytes(raw_, path, &data, &len), "DynMsg::get_bytes");
    std::vector<uint8_t> out;
    if (data && len > 0) {
      out.assign(data, data + len);
    }
    robot_bus_free_bytes(data, len);
    return out;
  }

  void set_bytes(const char *path, const std::vector<uint8_t> &value) {
    check(robot_bus_ros2_dyn_msg_set_bytes(raw_, path, value.data(), value.size()),
          "DynMsg::set_bytes");
  }

 private:
  RobotBusRos2DynMsg *raw_ = nullptr;
};

/// Custom topic mapper implemented in C++ (ROS DynamicMessage ↔ bus protobuf bytes).
///
/// Keep implementations thread-safe: callbacks may run on the ROS spin thread or bus
/// subscription thread.
class TopicMapper {
 public:
  virtual ~TopicMapper() = default;
  virtual const char *type_name() const = 0;
  virtual std::vector<uint8_t> ros_to_bus(const DynMsg &msg) = 0;
  virtual void bus_to_ros(const uint8_t *payload, size_t len, DynMsg &msg) = 0;
};

/// Builtin topic/service/action type tag: `.mapper(StdMsgsStringMapper{})`.
struct StdMsgsStringMapper {
  static constexpr const char *type_name = "std_msgs/msg/String";
};
struct SensorMsgsImageMapper {
  static constexpr const char *type_name = "sensor_msgs/msg/Image";
};
struct SensorMsgsImuMapper {
  static constexpr const char *type_name = "sensor_msgs/msg/Imu";
};
struct TriggerServiceMapper {
  static constexpr const char *type_name = "std_srvs/srv/Trigger";
};
struct SetBoolServiceMapper {
  static constexpr const char *type_name = "std_srvs/srv/SetBool";
};
struct FibonacciActionMapper {
  static constexpr const char *type_name = "example_interfaces/action/Fibonacci";
};

namespace detail {
template <typename T>
using BuiltinTypeName = decltype(T::type_name);
}  // namespace detail

class Ros2Bridge;
class Ros2BridgeBuilder;

namespace detail {

struct TopicMapperHolder {
  std::shared_ptr<TopicMapper> mapper;
};

inline int topic_mapper_ros_to_bus(const RobotBusRos2DynMsg *ros_msg, uint8_t **out_bus,
                                   size_t *out_len, void *user) {
  auto *holder = static_cast<TopicMapperHolder *>(user);
  try {
    DynMsg msg(ros_msg);
    std::vector<uint8_t> bytes = holder->mapper->ros_to_bus(msg);
    if (bytes.empty()) {
      *out_bus = nullptr;
      *out_len = 0;
      return 0;
    }
    uint8_t *buf = robot_bus_alloc_bytes(bytes.size());
    if (!buf) {
      robot_bus_set_error("robot_bus_alloc_bytes failed");
      return -1;
    }
    std::memcpy(buf, bytes.data(), bytes.size());
    *out_bus = buf;
    *out_len = bytes.size();
    return 0;
  } catch (const std::exception &e) {
    robot_bus_set_error(e.what());
    return -1;
  } catch (...) {
    robot_bus_set_error("TopicMapper::ros_to_bus failed");
    return -1;
  }
}

inline int topic_mapper_bus_to_ros(const uint8_t *bus_payload, size_t bus_len,
                                  RobotBusRos2DynMsg *ros_msg, void *user) {
  auto *holder = static_cast<TopicMapperHolder *>(user);
  try {
    DynMsg msg(ros_msg);
    holder->mapper->bus_to_ros(bus_payload, bus_len, msg);
    return 0;
  } catch (const std::exception &e) {
    robot_bus_set_error(e.what());
    return -1;
  } catch (...) {
    robot_bus_set_error("TopicMapper::bus_to_ros failed");
    return -1;
  }
}

inline void topic_mapper_drop(void *user) { delete static_cast<TopicMapperHolder *>(user); }

}  // namespace detail

/// Intermediate topic route configuration before `.add()`.
class Ros2BridgeRoute {
 public:
  Ros2BridgeRoute(RobotBusRos2BridgeBuilder *b, std::string ros_topic, std::string bus_topic)
      : b_(b), ros_topic_(std::move(ros_topic)), bus_topic_(std::move(bus_topic)) {}

  ~Ros2BridgeRoute() { robot_bus_ros2_bridge_builder_free(b_); }

  Ros2BridgeRoute(const Ros2BridgeRoute &) = delete;
  Ros2BridgeRoute &operator=(const Ros2BridgeRoute &) = delete;

  Ros2BridgeRoute(Ros2BridgeRoute &&o) noexcept
      : b_(o.b_),
        ros_topic_(std::move(o.ros_topic_)),
        bus_topic_(std::move(o.bus_topic_)),
        type_(std::move(o.type_)),
        custom_(std::move(o.custom_)),
        direction_(o.direction_) {
    o.b_ = nullptr;
  }

  Ros2BridgeRoute &operator=(Ros2BridgeRoute &&o) noexcept {
    if (this != &o) {
      robot_bus_ros2_bridge_builder_free(b_);
      b_ = o.b_;
      o.b_ = nullptr;
      ros_topic_ = std::move(o.ros_topic_);
      bus_topic_ = std::move(o.bus_topic_);
      type_ = std::move(o.type_);
      custom_ = std::move(o.custom_);
      direction_ = o.direction_;
    }
    return *this;
  }

  /// Builtin type string (same as Rust lookup). Prefer `.mapper(StdMsgsStringMapper{})`.
  Ros2BridgeRoute &&type_name(std::string type) && {
    type_ = std::move(type);
    custom_.reset();
    return std::move(*this);
  }

  /// Builtin tag (e.g. `StdMsgsStringMapper{}`) or custom `shared_ptr<TopicMapper>`.
  template <typename T, typename = detail::BuiltinTypeName<T>>
  Ros2BridgeRoute &&mapper(T) && {
    type_ = T::type_name;
    custom_.reset();
    return std::move(*this);
  }

  Ros2BridgeRoute &&mapper(std::shared_ptr<TopicMapper> m) && {
    if (!m) {
      throw Error("ros2 bridge route: null TopicMapper");
    }
    custom_ = std::move(m);
    type_.clear();
    return std::move(*this);
  }

  Ros2BridgeRoute &&direction(Direction d) && {
    direction_ = d;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  friend class Ros2BridgeBuilder;
  RobotBusRos2BridgeBuilder *b_ = nullptr;
  std::string ros_topic_;
  std::string bus_topic_;
  std::string type_;
  std::shared_ptr<TopicMapper> custom_;
  Direction direction_ = Direction::Ros2ToBus;
};

/// Intermediate service route configuration before `.add()`.
class Ros2BridgeService {
 public:
  Ros2BridgeService(RobotBusRos2BridgeBuilder *b, std::string ros_service, std::string bus_service)
      : b_(b),
        ros_service_(std::move(ros_service)),
        bus_service_(std::move(bus_service)) {}

  ~Ros2BridgeService() { robot_bus_ros2_bridge_builder_free(b_); }

  Ros2BridgeService(const Ros2BridgeService &) = delete;
  Ros2BridgeService &operator=(const Ros2BridgeService &) = delete;

  Ros2BridgeService(Ros2BridgeService &&o) noexcept
      : b_(o.b_),
        ros_service_(std::move(o.ros_service_)),
        bus_service_(std::move(o.bus_service_)),
        type_(std::move(o.type_)),
        direction_(o.direction_) {
    o.b_ = nullptr;
  }

  Ros2BridgeService &operator=(Ros2BridgeService &&o) noexcept {
    if (this != &o) {
      robot_bus_ros2_bridge_builder_free(b_);
      b_ = o.b_;
      o.b_ = nullptr;
      ros_service_ = std::move(o.ros_service_);
      bus_service_ = std::move(o.bus_service_);
      type_ = std::move(o.type_);
      direction_ = o.direction_;
    }
    return *this;
  }

  Ros2BridgeService &&type_name(std::string type) && {
    type_ = std::move(type);
    return std::move(*this);
  }

  template <typename T, typename = detail::BuiltinTypeName<T>>
  Ros2BridgeService &&mapper(T) && {
    type_ = T::type_name;
    return std::move(*this);
  }

  Ros2BridgeService &&direction(Direction d) && {
    direction_ = d;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  friend class Ros2BridgeBuilder;
  RobotBusRos2BridgeBuilder *b_ = nullptr;
  std::string ros_service_;
  std::string bus_service_;
  std::string type_;
  Direction direction_ = Direction::Ros2ToBus;
};

/// Intermediate action route configuration before `.add()`.
class Ros2BridgeAction {
 public:
  Ros2BridgeAction(RobotBusRos2BridgeBuilder *b, std::string ros_action, std::string bus_action)
      : b_(b),
        ros_action_(std::move(ros_action)),
        bus_action_(std::move(bus_action)) {}

  ~Ros2BridgeAction() { robot_bus_ros2_bridge_builder_free(b_); }

  Ros2BridgeAction(const Ros2BridgeAction &) = delete;
  Ros2BridgeAction &operator=(const Ros2BridgeAction &) = delete;

  Ros2BridgeAction(Ros2BridgeAction &&o) noexcept
      : b_(o.b_),
        ros_action_(std::move(o.ros_action_)),
        bus_action_(std::move(o.bus_action_)),
        type_(std::move(o.type_)),
        direction_(o.direction_) {
    o.b_ = nullptr;
  }

  Ros2BridgeAction &operator=(Ros2BridgeAction &&o) noexcept {
    if (this != &o) {
      robot_bus_ros2_bridge_builder_free(b_);
      b_ = o.b_;
      o.b_ = nullptr;
      ros_action_ = std::move(o.ros_action_);
      bus_action_ = std::move(o.bus_action_);
      type_ = std::move(o.type_);
      direction_ = o.direction_;
    }
    return *this;
  }

  Ros2BridgeAction &&type_name(std::string type) && {
    type_ = std::move(type);
    return std::move(*this);
  }

  template <typename T, typename = detail::BuiltinTypeName<T>>
  Ros2BridgeAction &&mapper(T) && {
    type_ = T::type_name;
    return std::move(*this);
  }

  Ros2BridgeAction &&direction(Direction d) && {
    direction_ = d;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  friend class Ros2BridgeBuilder;
  RobotBusRos2BridgeBuilder *b_ = nullptr;
  std::string ros_action_;
  std::string bus_action_;
  std::string type_;
  Direction direction_ = Direction::Ros2ToBus;
};

/// Fluent builder matching Rust `Ros2Bridge::new(...).bus_tcp(...).route(...).add()`.
class Ros2BridgeBuilder {
 public:
  explicit Ros2BridgeBuilder(std::string name) {
    b_ = static_cast<RobotBusRos2BridgeBuilder *>(check_ptr(
        robot_bus_ros2_bridge_builder_new(name.c_str()), "Ros2BridgeBuilder"));
  }

  explicit Ros2BridgeBuilder(RobotBusRos2BridgeBuilder *raw) : b_(raw) {}

  ~Ros2BridgeBuilder() { robot_bus_ros2_bridge_builder_free(b_); }

  Ros2BridgeBuilder(const Ros2BridgeBuilder &) = delete;
  Ros2BridgeBuilder &operator=(const Ros2BridgeBuilder &) = delete;

  Ros2BridgeBuilder(Ros2BridgeBuilder &&o) noexcept : b_(o.b_) { o.b_ = nullptr; }

  Ros2BridgeBuilder &operator=(Ros2BridgeBuilder &&o) noexcept {
    if (this != &o) {
      robot_bus_ros2_bridge_builder_free(b_);
      b_ = o.b_;
      o.b_ = nullptr;
    }
    return *this;
  }

  Ros2BridgeBuilder &&bus_tcp(const std::string &host) && {
    check(robot_bus_ros2_bridge_builder_bus_tcp(b_, host.c_str()), "bus_tcp");
    return std::move(*this);
  }

  Ros2BridgeBuilder &&bus_ipc() && {
    check(robot_bus_ros2_bridge_builder_bus_ipc(b_), "bus_ipc");
    return std::move(*this);
  }

  Ros2BridgeBuilder &&bus_ipc_at(const std::string &dir) && {
    check(robot_bus_ros2_bridge_builder_bus_ipc_at(b_, dir.c_str()), "bus_ipc_at");
    return std::move(*this);
  }

  /// HTTP discover then TCP. Empty `api_url` → default; `timeout_secs <= 0` uses the
  /// default; empty broker_id = any.
  Ros2BridgeBuilder &&bus_discover(const std::string &api_url = {},
                                   double timeout_secs = 0.0,
                                   const std::string &broker_id = {}) && {
    check(robot_bus_ros2_bridge_builder_bus_discover(
              b_, api_url.empty() ? nullptr : api_url.c_str(), timeout_secs,
              broker_id.empty() ? nullptr : broker_id.c_str()),
          "bus_discover");
    return std::move(*this);
  }

  Ros2BridgeRoute route(std::string ros_topic, std::string bus_topic) && {
    RobotBusRos2BridgeBuilder *b = b_;
    b_ = nullptr;
    return Ros2BridgeRoute(b, std::move(ros_topic), std::move(bus_topic));
  }

  Ros2BridgeService service(std::string ros_service, std::string bus_service) && {
    RobotBusRos2BridgeBuilder *b = b_;
    b_ = nullptr;
    return Ros2BridgeService(b, std::move(ros_service), std::move(bus_service));
  }

  Ros2BridgeAction action(std::string ros_action, std::string bus_action) && {
    RobotBusRos2BridgeBuilder *b = b_;
    b_ = nullptr;
    return Ros2BridgeAction(b, std::move(ros_action), std::move(bus_action));
  }

  /// Imperative add (type_name e.g. `std_msgs/msg/String`).
  Ros2BridgeBuilder &&add_route(const std::string &ros_topic, const std::string &bus_topic,
                                const std::string &type_name,
                                Direction direction = Direction::Ros2ToBus) && {
    check(robot_bus_ros2_bridge_builder_add_route(b_, ros_topic.c_str(), bus_topic.c_str(),
                                                   type_name.c_str(),
                                                   static_cast<int>(direction)),
          "add_route");
    return std::move(*this);
  }

  Ros2BridgeBuilder &&add_route(const std::string &ros_topic, const std::string &bus_topic,
                                std::shared_ptr<TopicMapper> mapper,
                                Direction direction = Direction::Ros2ToBus) && {
    if (!mapper) {
      throw Error("add_route: null TopicMapper");
    }
    auto *holder = new detail::TopicMapperHolder{std::move(mapper)};
    RobotBusRos2TopicMapperVtable vt{};
    vt.type_name = holder->mapper->type_name();
    vt.ros_to_bus = detail::topic_mapper_ros_to_bus;
    vt.bus_to_ros = detail::topic_mapper_bus_to_ros;
    vt.drop_user = detail::topic_mapper_drop;
    vt.user = holder;
    int rc = robot_bus_ros2_bridge_builder_add_route_mapper(
        b_, ros_topic.c_str(), bus_topic.c_str(), &vt, static_cast<int>(direction));
    if (rc != 0) {
      detail::topic_mapper_drop(holder);
      check(rc, "add_route_mapper");
    }
    return std::move(*this);
  }

  /// Imperative service add (`std_srvs/srv/Trigger` or `SetBool`).
  Ros2BridgeBuilder &&add_service(const std::string &ros_service, const std::string &bus_service,
                                  const std::string &type_name,
                                  Direction direction = Direction::Ros2ToBus) && {
    check(robot_bus_ros2_bridge_builder_add_service(b_, ros_service.c_str(), bus_service.c_str(),
                                                     type_name.c_str(),
                                                     static_cast<int>(direction)),
          "add_service");
    return std::move(*this);
  }

  /// Imperative action add (`example_interfaces/action/Fibonacci`).
  Ros2BridgeBuilder &&add_action(const std::string &ros_action, const std::string &bus_action,
                                 const std::string &type_name,
                                 Direction direction = Direction::Ros2ToBus) && {
    check(robot_bus_ros2_bridge_builder_add_action(b_, ros_action.c_str(), bus_action.c_str(),
                                                    type_name.c_str(),
                                                    static_cast<int>(direction)),
          "add_action");
    return std::move(*this);
  }

  Ros2Bridge build() &&;

 private:
  friend class Ros2BridgeRoute;
  friend class Ros2BridgeService;
  friend class Ros2BridgeAction;
  friend class Ros2Bridge;
  RobotBusRos2BridgeBuilder *b_ = nullptr;
};

/// In-process ROS 2 ↔ robot-bus topic/service/action bridge (`feature = "ros2"` / Linux ros2 packages).
class Ros2Bridge {
 public:
  static Ros2BridgeBuilder New(std::string name) {
    return Ros2BridgeBuilder(std::move(name));
  }

  static Ros2Bridge from_yaml(const std::string &path) {
    return Ros2Bridge(static_cast<RobotBusRos2Bridge *>(check_ptr(
        robot_bus_ros2_bridge_from_yaml(path.c_str()), "Ros2Bridge::from_yaml")));
  }

  ~Ros2Bridge() { robot_bus_ros2_bridge_free(b_); }

  Ros2Bridge(const Ros2Bridge &) = delete;
  Ros2Bridge &operator=(const Ros2Bridge &) = delete;

  Ros2Bridge(Ros2Bridge &&o) noexcept : b_(o.b_) { o.b_ = nullptr; }

  Ros2Bridge &operator=(Ros2Bridge &&o) noexcept {
    if (this != &o) {
      robot_bus_ros2_bridge_free(b_);
      b_ = o.b_;
      o.b_ = nullptr;
    }
    return *this;
  }

  void spin() { check(robot_bus_ros2_bridge_spin(b_), "Ros2Bridge::spin"); }

  void spin_once(double timeout_secs = 0.01) {
    check(robot_bus_ros2_bridge_spin_once(b_, timeout_secs), "Ros2Bridge::spin_once");
  }

  RobotBusRos2Bridge *raw() { return b_; }

 private:
  friend class Ros2BridgeBuilder;
  explicit Ros2Bridge(RobotBusRos2Bridge *raw) : b_(raw) {}

  RobotBusRos2Bridge *b_ = nullptr;
};

inline Ros2BridgeBuilder Ros2BridgeRoute::add() && {
  if (custom_) {
    auto *holder = new detail::TopicMapperHolder{std::move(custom_)};
    RobotBusRos2TopicMapperVtable vt{};
    vt.type_name = holder->mapper->type_name();
    vt.ros_to_bus = detail::topic_mapper_ros_to_bus;
    vt.bus_to_ros = detail::topic_mapper_bus_to_ros;
    vt.drop_user = detail::topic_mapper_drop;
    vt.user = holder;
    int rc = robot_bus_ros2_bridge_builder_add_route_mapper(
        b_, ros_topic_.c_str(), bus_topic_.c_str(), &vt, static_cast<int>(direction_));
    if (rc != 0) {
      detail::topic_mapper_drop(holder);
      check(rc, "add_route_mapper");
    }
  } else {
    if (type_.empty()) {
      throw Error(
          "ros2 bridge route: call .mapper(...) or .type_name(\"pkg/msg/Type\") before .add()");
    }
    check(robot_bus_ros2_bridge_builder_add_route(b_, ros_topic_.c_str(), bus_topic_.c_str(),
                                                   type_.c_str(), static_cast<int>(direction_)),
          "add_route");
  }
  RobotBusRos2BridgeBuilder *b = b_;
  b_ = nullptr;
  return Ros2BridgeBuilder(b);
}

inline Ros2BridgeBuilder Ros2BridgeService::add() && {
  if (type_.empty()) {
    throw Error("ros2 bridge service: call .mapper(...) or .type_name(\"pkg/srv/Type\") before .add()");
  }
  check(robot_bus_ros2_bridge_builder_add_service(b_, ros_service_.c_str(), bus_service_.c_str(),
                                                   type_.c_str(), static_cast<int>(direction_)),
        "add_service");
  RobotBusRos2BridgeBuilder *b = b_;
  b_ = nullptr;
  return Ros2BridgeBuilder(b);
}

inline Ros2BridgeBuilder Ros2BridgeAction::add() && {
  if (type_.empty()) {
    throw Error("ros2 bridge action: call .mapper(...) or .type_name(\"pkg/action/Type\") before .add()");
  }
  check(robot_bus_ros2_bridge_builder_add_action(b_, ros_action_.c_str(), bus_action_.c_str(),
                                                  type_.c_str(), static_cast<int>(direction_)),
        "add_action");
  RobotBusRos2BridgeBuilder *b = b_;
  b_ = nullptr;
  return Ros2BridgeBuilder(b);
}

inline Ros2Bridge Ros2BridgeBuilder::build() && {
  RobotBusRos2Bridge *bridge = static_cast<RobotBusRos2Bridge *>(
      check_ptr(robot_bus_ros2_bridge_builder_build(b_), "Ros2BridgeBuilder::build"));
  return Ros2Bridge(bridge);
}

}  // namespace robot_bus
