#pragma once

#include <robot_bus/node.hpp>

#include <cstdint>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

#ifdef ROBOT_BUS_HAS_ROS2
#include <mutex>
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

#ifdef ROBOT_BUS_HAS_ROS2
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
  rclcpp::CallbackGroup::SharedPtr callback_group;
  std::vector<std::shared_ptr<void>> &keep_alive;

  template <typename T>
  void retain(std::shared_ptr<T> p) {
    keep_alive.push_back(std::shared_ptr<void>(std::move(p)));
  }
};
#endif  // ROBOT_BUS_HAS_ROS2

/// Topic codec. Builtins are ZSTs; custom mappers override `attach` (ROS2 builds).
class TopicMapper {
 public:
  virtual ~TopicMapper() = default;
  virtual const char *type_name() const = 0;
  /// Builtins and [`TypedTopicMapper`] support opt-in `.lazy()`. Attach-only
  /// custom mappers return false (`.lazy().add()` throws).
  virtual bool supports_lazy() const { return false; }
#ifdef ROBOT_BUS_HAS_ROS2
  /// Wire this topic. Default throws — builtins use the library path instead.
  virtual void attach(TopicWireContext &ctx);
  /// Create a ROS subscription that forwards onto `bus_pub`. Used by `.lazy()`.
  virtual rclcpp::SubscriptionBase::SharedPtr create_ros2_to_bus_subscription(
      rclcpp::Node::SharedPtr ros_node, const std::string &ros_topic,
      std::shared_ptr<TopicPublisher> bus_pub, std::shared_ptr<std::mutex> mtx,
      const rclcpp::QoS &qos);
#endif
};

/// Service codec. Custom: override `attach` with concrete `create_service<T>`.
class ServiceMapper {
 public:
  virtual ~ServiceMapper() = default;
  virtual const char *type_name() const = 0;
#ifdef ROBOT_BUS_HAS_ROS2
  virtual void attach(ServiceWireContext &ctx);
#endif
};

/// Action codec. Custom: override `attach` with concrete action types.
class ActionMapper {
 public:
  virtual ~ActionMapper() = default;
  virtual const char *type_name() const = 0;
#ifdef ROBOT_BUS_HAS_ROS2
  virtual void attach(ActionWireContext &ctx);
#endif
};

/// Builtin: `std_msgs/msg/String` ↔ bus `std_msgs.msg.v1.String`.
struct StdMsgsStringMapper : TopicMapper {
  const char *type_name() const override { return "std_msgs/msg/String"; }
};

/// Builtin: `sensor_msgs/msg/Image` ↔ bus `sensor_msgs.msg.v1.Image`.
struct SensorMsgsImageMapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/Image"; }
};

/// Builtin: `std_srvs/srv/Trigger`.
struct TriggerServiceMapper : ServiceMapper {
  const char *type_name() const override { return "std_srvs/srv/Trigger"; }
};

/// Builtin: `std_srvs/srv/SetBool`.
struct SetBoolServiceMapper : ServiceMapper {
  const char *type_name() const override { return "std_srvs/srv/SetBool"; }
};

/// Builtin: `example_interfaces/action/Fibonacci`.
struct FibonacciActionMapper : ActionMapper {
  const char *type_name() const override { return "example_interfaces/action/Fibonacci"; }
};

#ifdef ROBOT_BUS_HAS_ROS2
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

class Ros2Bridge;
class Ros2BridgeBuilder;
class Ros2BridgeRoute;
class Ros2BridgeServiceRoute;
class Ros2BridgeActionRoute;

namespace detail {

enum class TopicBuiltin { StdMsgsString, SensorMsgsImage };
enum class ServiceBuiltin { Trigger, SetBool };
enum class ActionBuiltin { Fibonacci };

struct TopicRouteQos {
  std::optional<int32_t> depth;
  bool best_effort = false;
  bool sensor_data = false;
};

struct TopicRouteSpec {
  std::string ros_topic;
  std::string bus_topic;
  Direction direction = Direction::Ros2ToBus;
  TopicBuiltin builtin = TopicBuiltin::StdMsgsString;
  std::shared_ptr<TopicMapper> custom;
  bool lazy = false;
  TopicRouteQos qos;
  bool is_custom() const { return static_cast<bool>(custom); }
};

struct ServiceRouteSpec {
  std::string ros_service;
  std::string bus_service;
  Direction direction = Direction::Ros2ToBus;
  double timeout_secs = kServiceCallTimeoutSecs;
  ServiceBuiltin builtin = ServiceBuiltin::Trigger;
  std::shared_ptr<ServiceMapper> custom;
  bool is_custom() const { return static_cast<bool>(custom); }
};

struct ActionRouteSpec {
  std::string ros_action;
  std::string bus_action;
  Direction direction = Direction::Ros2ToBus;
  double timeout_secs = kActionCallTimeoutSecs;
  ActionBuiltin builtin = ActionBuiltin::Fibonacci;
  std::shared_ptr<ActionMapper> custom;
  bool is_custom() const { return static_cast<bool>(custom); }
};

enum class BusTransportKind { Tcp, Ipc, IpcAt, Discover };

struct BusTransport {
  BusTransportKind kind = BusTransportKind::Tcp;
  std::string host = "localhost";
  std::string ipc_path;
  std::string api_url;
  double discover_timeout_secs = 0.0;
  std::string broker_id;
};

struct BuilderState {
  std::string name;
  BusTransport bus;
  std::vector<TopicRouteSpec> routes;
  std::vector<ServiceRouteSpec> services;
  std::vector<ActionRouteSpec> actions;
};

}  // namespace detail

/// Intermediate topic route: `.mapper(...).direction(...).add()`.
class Ros2BridgeRoute {
 public:
  Ros2BridgeRoute(std::shared_ptr<detail::BuilderState> state, std::string ros_topic,
                  std::string bus_topic)
      : state_(std::move(state)),
        ros_topic_(std::move(ros_topic)),
        bus_topic_(std::move(bus_topic)) {}

  Ros2BridgeRoute(const Ros2BridgeRoute &) = delete;
  Ros2BridgeRoute &operator=(const Ros2BridgeRoute &) = delete;
  Ros2BridgeRoute(Ros2BridgeRoute &&) noexcept = default;
  Ros2BridgeRoute &operator=(Ros2BridgeRoute &&) noexcept = default;

  Ros2BridgeRoute &&mapper(StdMsgsStringMapper) && {
    builtin_ = detail::TopicBuiltin::StdMsgsString;
    custom_.reset();
    mapper_set_ = true;
    return std::move(*this);
  }

  Ros2BridgeRoute &&mapper(SensorMsgsImageMapper) && {
    builtin_ = detail::TopicBuiltin::SensorMsgsImage;
    custom_.reset();
    mapper_set_ = true;
    return std::move(*this);
  }

  /// Custom topic mapper (override `attach` under `ROBOT_BUS_HAS_ROS2`).
  Ros2BridgeRoute &&mapper(std::shared_ptr<TopicMapper> mapper) && {
    if (!mapper) {
      throw Error("ros2 bridge route: mapper shared_ptr must not be null");
    }
    custom_ = std::move(mapper);
    mapper_set_ = true;
    return std::move(*this);
  }

  Ros2BridgeRoute &&direction(Direction d) && {
    direction_ = d;
    return std::move(*this);
  }

  /// Opt-in lazy ROS 2 subscription for **ROS2→bus** topics only.
  /// Default is eager (`build()` creates the ROS subscription immediately).
  Ros2BridgeRoute &&lazy() && {
    lazy_ = true;
    return std::move(*this);
  }

  /// ROS KeepLast(`n`) plus bus topic HWM `n`.
  Ros2BridgeRoute &&qos_depth(int32_t n) && {
    qos_.depth = n;
    return std::move(*this);
  }

  /// ROS reliability best-effort.
  Ros2BridgeRoute &&best_effort() && {
    qos_.best_effort = true;
    return std::move(*this);
  }

  /// Best-effort KeepLast(5) on ROS (`SensorDataQoS`) and bus depth 5.
  Ros2BridgeRoute &&sensor_data() && {
    qos_.sensor_data = true;
    qos_.depth = 5;
    qos_.best_effort = true;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_topic_;
  std::string bus_topic_;
  Direction direction_ = Direction::Ros2ToBus;
  detail::TopicBuiltin builtin_ = detail::TopicBuiltin::StdMsgsString;
  std::shared_ptr<TopicMapper> custom_;
  bool mapper_set_ = false;
  bool lazy_ = false;
  detail::TopicRouteQos qos_;
};

/// Intermediate service route: `.mapper(...).timeout(...).direction(...).add()`.
class Ros2BridgeServiceRoute {
 public:
  Ros2BridgeServiceRoute(std::shared_ptr<detail::BuilderState> state, std::string ros_service,
                         std::string bus_service)
      : state_(std::move(state)),
        ros_service_(std::move(ros_service)),
        bus_service_(std::move(bus_service)) {}

  Ros2BridgeServiceRoute(const Ros2BridgeServiceRoute &) = delete;
  Ros2BridgeServiceRoute &operator=(const Ros2BridgeServiceRoute &) = delete;
  Ros2BridgeServiceRoute(Ros2BridgeServiceRoute &&) noexcept = default;
  Ros2BridgeServiceRoute &operator=(Ros2BridgeServiceRoute &&) noexcept = default;

  Ros2BridgeServiceRoute &&mapper(TriggerServiceMapper) && {
    builtin_ = detail::ServiceBuiltin::Trigger;
    custom_.reset();
    mapper_set_ = true;
    return std::move(*this);
  }

  Ros2BridgeServiceRoute &&mapper(SetBoolServiceMapper) && {
    builtin_ = detail::ServiceBuiltin::SetBool;
    custom_.reset();
    mapper_set_ = true;
    return std::move(*this);
  }

  /// Custom service mapper (override `attach` with concrete ROS srv type).
  Ros2BridgeServiceRoute &&mapper(std::shared_ptr<ServiceMapper> mapper) && {
    if (!mapper) {
      throw Error("ros2 bridge service: mapper shared_ptr must not be null");
    }
    custom_ = std::move(mapper);
    mapper_set_ = true;
    return std::move(*this);
  }

  Ros2BridgeServiceRoute &&direction(Direction d) && {
    direction_ = d;
    return std::move(*this);
  }

  Ros2BridgeServiceRoute &&timeout(double timeout_secs) && {
    timeout_secs_ = timeout_secs;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_service_;
  std::string bus_service_;
  Direction direction_ = Direction::Ros2ToBus;
  double timeout_secs_ = kServiceCallTimeoutSecs;
  detail::ServiceBuiltin builtin_ = detail::ServiceBuiltin::Trigger;
  std::shared_ptr<ServiceMapper> custom_;
  bool mapper_set_ = false;
};

/// Intermediate action route: `.mapper(...).timeout(...).direction(...).add()`.
class Ros2BridgeActionRoute {
 public:
  Ros2BridgeActionRoute(std::shared_ptr<detail::BuilderState> state, std::string ros_action,
                        std::string bus_action)
      : state_(std::move(state)),
        ros_action_(std::move(ros_action)),
        bus_action_(std::move(bus_action)) {}

  Ros2BridgeActionRoute(const Ros2BridgeActionRoute &) = delete;
  Ros2BridgeActionRoute &operator=(const Ros2BridgeActionRoute &) = delete;
  Ros2BridgeActionRoute(Ros2BridgeActionRoute &&) noexcept = default;
  Ros2BridgeActionRoute &operator=(Ros2BridgeActionRoute &&) noexcept = default;

  Ros2BridgeActionRoute &&mapper(FibonacciActionMapper) && {
    builtin_ = detail::ActionBuiltin::Fibonacci;
    custom_.reset();
    mapper_set_ = true;
    return std::move(*this);
  }

  /// Custom action mapper (override `attach` with concrete ROS action type).
  Ros2BridgeActionRoute &&mapper(std::shared_ptr<ActionMapper> mapper) && {
    if (!mapper) {
      throw Error("ros2 bridge action: mapper shared_ptr must not be null");
    }
    custom_ = std::move(mapper);
    mapper_set_ = true;
    return std::move(*this);
  }

  Ros2BridgeActionRoute &&direction(Direction d) && {
    direction_ = d;
    return std::move(*this);
  }

  Ros2BridgeActionRoute &&timeout(double timeout_secs) && {
    timeout_secs_ = timeout_secs;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_action_;
  std::string bus_action_;
  Direction direction_ = Direction::Ros2ToBus;
  double timeout_secs_ = kActionCallTimeoutSecs;
  detail::ActionBuiltin builtin_ = detail::ActionBuiltin::Fibonacci;
  std::shared_ptr<ActionMapper> custom_;
  bool mapper_set_ = false;
};

/// Fluent builder: `Ros2Bridge::New(name).bus_tcp(...).route(...).mapper(...).add().build()`.
class Ros2BridgeBuilder {
 public:
  explicit Ros2BridgeBuilder(std::string name)
      : state_(std::make_shared<detail::BuilderState>()) {
    state_->name = std::move(name);
  }

  explicit Ros2BridgeBuilder(std::shared_ptr<detail::BuilderState> state)
      : state_(std::move(state)) {}

  Ros2BridgeBuilder(const Ros2BridgeBuilder &) = delete;
  Ros2BridgeBuilder &operator=(const Ros2BridgeBuilder &) = delete;
  Ros2BridgeBuilder(Ros2BridgeBuilder &&) noexcept = default;
  Ros2BridgeBuilder &operator=(Ros2BridgeBuilder &&) noexcept = default;

  Ros2BridgeBuilder &&bus_tcp(const std::string &host = "localhost") && {
    state_->bus.kind = detail::BusTransportKind::Tcp;
    state_->bus.host = host;
    return std::move(*this);
  }

  Ros2BridgeBuilder &&bus_ipc() && {
    state_->bus.kind = detail::BusTransportKind::Ipc;
    state_->bus.ipc_path.clear();
    return std::move(*this);
  }

  Ros2BridgeBuilder &&bus_ipc_at(const std::string &dir) && {
    state_->bus.kind = detail::BusTransportKind::IpcAt;
    state_->bus.ipc_path = dir;
    return std::move(*this);
  }

  /// HTTP discover then TCP. Empty `api_url` → default; `timeout_secs <= 0` uses default;
  /// empty `broker_id` = any.
  Ros2BridgeBuilder &&bus_discover(const std::string &api_url = {}, double timeout_secs = 0.0,
                                   const std::string &broker_id = {}) && {
    state_->bus.kind = detail::BusTransportKind::Discover;
    state_->bus.api_url = api_url;
    state_->bus.discover_timeout_secs = timeout_secs;
    state_->bus.broker_id = broker_id;
    return std::move(*this);
  }

  Ros2BridgeRoute route(std::string ros_topic, std::string bus_topic) && {
    auto state = std::move(state_);
    return Ros2BridgeRoute(std::move(state), std::move(ros_topic), std::move(bus_topic));
  }

  Ros2BridgeServiceRoute service(std::string ros_service, std::string bus_service) && {
    auto state = std::move(state_);
    return Ros2BridgeServiceRoute(std::move(state), std::move(ros_service),
                                  std::move(bus_service));
  }

  Ros2BridgeActionRoute action(std::string ros_action, std::string bus_action) && {
    auto state = std::move(state_);
    return Ros2BridgeActionRoute(std::move(state), std::move(ros_action),
                                 std::move(bus_action));
  }

  Ros2Bridge build() &&;

 private:
  friend class Ros2BridgeRoute;
  friend class Ros2BridgeServiceRoute;
  friend class Ros2BridgeActionRoute;
  std::shared_ptr<detail::BuilderState> state_;
};

/// In-process rclcpp ↔ robot-bus bridge (`robot_bus_ros2_bridge` / `ROBOT_BUS_HAS_ROS2`).
class Ros2Bridge {
 public:
  static Ros2BridgeBuilder New(std::string name) {
    return Ros2BridgeBuilder(std::move(name));
  }

#ifdef ROBOT_BUS_HAS_ROS2
  ~Ros2Bridge();
  Ros2Bridge(const Ros2Bridge &) = delete;
  Ros2Bridge &operator=(const Ros2Bridge &) = delete;
  Ros2Bridge(Ros2Bridge &&) noexcept;
  Ros2Bridge &operator=(Ros2Bridge &&) noexcept;

  void spin();
  void spin_once(double timeout_secs = 0.01);
  /// True when this bridge currently holds a ROS subscription for `bus_topic`.
  bool has_ros_subscription(const std::string &bus_topic) const;

 private:
  friend class Ros2BridgeBuilder;
  struct Impl;
  explicit Ros2Bridge(std::unique_ptr<Impl> impl);
  std::unique_ptr<Impl> impl_;
#else
  Ros2Bridge() = delete;
#endif
};

inline Ros2BridgeBuilder Ros2BridgeRoute::add() && {
  if (!mapper_set_) {
    throw Error(
        "ros2 bridge route: call .mapper(...) before .add() "
        "(builtin ZST or std::shared_ptr<TopicMapper>)");
  }
  if (lazy_ && direction_ != Direction::Ros2ToBus) {
    throw Error("ros2 bridge route: .lazy() is only valid for Direction::Ros2ToBus");
  }
  if (lazy_ && custom_ && !custom_->supports_lazy()) {
    throw Error(
        "ros2 bridge route: .lazy() is not supported for this custom TopicMapper "
        "(attach-only); use TypedTopicMapper");
  }
  detail::TopicRouteSpec spec;
  spec.ros_topic = std::move(ros_topic_);
  spec.bus_topic = std::move(bus_topic_);
  spec.direction = direction_;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  spec.lazy = lazy_;
  spec.qos = qos_;
  state_->routes.push_back(std::move(spec));
  return Ros2BridgeBuilder(std::move(state_));
}

inline Ros2BridgeBuilder Ros2BridgeServiceRoute::add() && {
  if (!mapper_set_) {
    throw Error(
        "ros2 bridge service: call .mapper(...) before .add() "
        "(builtin ZST or std::shared_ptr<ServiceMapper>)");
  }
  detail::ServiceRouteSpec spec;
  spec.ros_service = std::move(ros_service_);
  spec.bus_service = std::move(bus_service_);
  spec.direction = direction_;
  spec.timeout_secs = timeout_secs_;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  state_->services.push_back(std::move(spec));
  return Ros2BridgeBuilder(std::move(state_));
}

inline Ros2BridgeBuilder Ros2BridgeActionRoute::add() && {
  if (!mapper_set_) {
    throw Error(
        "ros2 bridge action: call .mapper(...) before .add() "
        "(FibonacciActionMapper{} or std::shared_ptr<ActionMapper>)");
  }
  detail::ActionRouteSpec spec;
  spec.ros_action = std::move(ros_action_);
  spec.bus_action = std::move(bus_action_);
  spec.direction = direction_;
  spec.timeout_secs = timeout_secs_;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  state_->actions.push_back(std::move(spec));
  return Ros2BridgeBuilder(std::move(state_));
}

#ifndef ROBOT_BUS_HAS_ROS2
inline Ros2Bridge Ros2BridgeBuilder::build() && {
  (void)state_;
  throw Error(kRos2BridgeUnavailable);
}
#endif

}  // namespace robot_bus
