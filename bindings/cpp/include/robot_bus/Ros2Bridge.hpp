#pragma once

#include <robot_bus.h>

#include <robot_bus/Node.hpp>

#include <string>
#include <utility>

namespace robot_bus {

/// Topic / service / action bridge direction (ROS 2 ↔ robot-bus).
enum class Ros2Direction {
  RosToBus = ROBOT_BUS_ROS2_DIR_ROS_TO_BUS,
  BusToRos = ROBOT_BUS_ROS2_DIR_BUS_TO_ROS,
  Both = ROBOT_BUS_ROS2_DIR_BOTH,
};

/// True if `librobot_bus` was built with `--features ros2`.
inline bool ros2_available() { return robot_bus_ros2_available() != 0; }

class Ros2Bridge;
class Ros2BridgeBuilder;

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
      direction_ = o.direction_;
    }
    return *this;
  }

  Ros2BridgeRoute &&string() && {
    type_ = "std_msgs/msg/String";
    return std::move(*this);
  }

  Ros2BridgeRoute &&imu() && {
    type_ = "sensor_msgs/msg/Imu";
    return std::move(*this);
  }

  Ros2BridgeRoute &&image() && {
    type_ = "sensor_msgs/msg/Image";
    return std::move(*this);
  }

  Ros2BridgeRoute &&compressed_video() && {
    type_ = "foxglove_msgs/msg/CompressedVideo";
    return std::move(*this);
  }

  Ros2BridgeRoute &&type_name(std::string type) && {
    type_ = std::move(type);
    return std::move(*this);
  }

  Ros2BridgeRoute &&direction(Ros2Direction d) && {
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
  Ros2Direction direction_ = Ros2Direction::Both;
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

  Ros2BridgeService &&trigger() && {
    type_ = "std_srvs/srv/Trigger";
    return std::move(*this);
  }

  Ros2BridgeService &&set_bool() && {
    type_ = "std_srvs/srv/SetBool";
    return std::move(*this);
  }

  Ros2BridgeService &&type_name(std::string type) && {
    type_ = std::move(type);
    return std::move(*this);
  }

  /// Services only support RosToBus / BusToRos (not Both).
  Ros2BridgeService &&direction(Ros2Direction d) && {
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
  Ros2Direction direction_ = Ros2Direction::RosToBus;
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

  Ros2BridgeAction &&fibonacci() && {
    type_ = "example_interfaces/action/Fibonacci";
    return std::move(*this);
  }

  Ros2BridgeAction &&type_name(std::string type) && {
    type_ = std::move(type);
    return std::move(*this);
  }

  /// Actions only support RosToBus / BusToRos (not Both).
  Ros2BridgeAction &&direction(Ros2Direction d) && {
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
  Ros2Direction direction_ = Ros2Direction::RosToBus;
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

  /// UDP discover then TCP. `timeout_secs <= 0` uses the default; empty broker_id = any.
  Ros2BridgeBuilder &&bus_discover(uint32_t domain_id, double timeout_secs = 0.0,
                                   const std::string &broker_id = {}) && {
    check(robot_bus_ros2_bridge_builder_bus_discover(
              b_, domain_id, timeout_secs,
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
                                Ros2Direction direction = Ros2Direction::Both) && {
    check(robot_bus_ros2_bridge_builder_add_route(b_, ros_topic.c_str(), bus_topic.c_str(),
                                                   type_name.c_str(),
                                                   static_cast<int>(direction)),
          "add_route");
    return std::move(*this);
  }

  /// Imperative service add (`std_srvs/srv/Trigger` or `SetBool`; not Both).
  Ros2BridgeBuilder &&add_service(const std::string &ros_service, const std::string &bus_service,
                                  const std::string &type_name,
                                  Ros2Direction direction = Ros2Direction::RosToBus) && {
    check(robot_bus_ros2_bridge_builder_add_service(b_, ros_service.c_str(), bus_service.c_str(),
                                                     type_name.c_str(),
                                                     static_cast<int>(direction)),
          "add_service");
    return std::move(*this);
  }

  /// Imperative action add (`example_interfaces/action/Fibonacci`; not Both).
  Ros2BridgeBuilder &&add_action(const std::string &ros_action, const std::string &bus_action,
                                 const std::string &type_name,
                                 Ros2Direction direction = Ros2Direction::RosToBus) && {
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
  if (type_.empty()) {
    throw Error(
        "ros2 bridge route: call .type_name(...) or .string()/.imu()/.image() before .add()");
  }
  check(robot_bus_ros2_bridge_builder_add_route(b_, ros_topic_.c_str(), bus_topic_.c_str(),
                                                 type_.c_str(), static_cast<int>(direction_)),
        "add_route");
  RobotBusRos2BridgeBuilder *b = b_;
  b_ = nullptr;
  return Ros2BridgeBuilder(b);
}

inline Ros2BridgeBuilder Ros2BridgeService::add() && {
  if (type_.empty()) {
    throw Error("ros2 bridge service: call .trigger() or .set_bool() before .add()");
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
    throw Error("ros2 bridge action: call .fibonacci() before .add()");
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
