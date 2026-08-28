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

/// KeepLast depth plus reliability. Same type on **ROS** endpoints for topics,
/// services, and actions. Bus only uses it on **topics** (must be `.best_effort()`);
/// service / action bus names have no QoS.
class TopicQos {
 public:
  static TopicQosKeepLast keep_last(int32_t depth) { return TopicQosKeepLast(depth); }
  int32_t depth() const { return depth_; }
  bool is_best_effort() const { return best_effort_; }
  bool is_reliable() const { return !best_effort_; }

 private:
  friend class TopicQosKeepLast;
  TopicQos(int32_t depth, bool best_effort) : depth_(depth), best_effort_(best_effort) {}
  int32_t depth_;
  bool best_effort_;
};

inline TopicQos TopicQosKeepLast::reliable() const { return TopicQos(depth_, false); }
inline TopicQos TopicQosKeepLast::best_effort() const { return TopicQos(depth_, true); }

inline void require_bus_best_effort(const TopicQos &qos) {
  if (qos.is_reliable()) {
    throw Error(
        "ros2 bridge: bus TopicQos must be .best_effort() "
        "(PUB/SUB has no reliable delivery)");
  }
}

#ifdef ROBOT_BUS_HAS_ROS2
inline rmw_qos_profile_t apply_keep_last_reliability(rmw_qos_profile_t base,
                                                     const TopicQos &qos) {
  base.history = RMW_QOS_POLICY_HISTORY_KEEP_LAST;
  base.depth = static_cast<size_t>(qos.depth() < 0 ? 0 : qos.depth());
  base.reliability = qos.is_best_effort() ? RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT
                                          : RMW_QOS_POLICY_RELIABILITY_RELIABLE;
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
class Ros2BridgeFromRos;
class Ros2BridgeFromRosToBus;
class Ros2BridgeRos2ToBusReady;
class Ros2BridgeFromBus;
class Ros2BridgeFromBusToRos;
class Ros2BridgeBusToRosReady;
class Ros2BridgeService;
class Ros2BridgeServiceFromRos;
class Ros2BridgeServiceFromBus;
class Ros2BridgeServicePair;
class Ros2BridgeServiceReady;
class Ros2BridgeAction;
class Ros2BridgeActionFromRos;
class Ros2BridgeActionFromBus;
class Ros2BridgeActionPair;
class Ros2BridgeActionReady;

namespace detail {

enum class TopicBuiltin { StdMsgsString, SensorMsgsImage };
enum class ServiceBuiltin { Trigger, SetBool };
enum class ActionBuiltin { Fibonacci };

struct TopicRouteSpec {
  std::string ros_topic;
  std::string bus_topic;
  Direction direction = Direction::Ros2ToBus;
  TopicBuiltin builtin = TopicBuiltin::StdMsgsString;
  std::shared_ptr<TopicMapper> custom;
  bool lazy = false;
  TopicQos ros_qos = TopicQos::keep_last(10).reliable();
  TopicQos bus_qos = TopicQos::keep_last(8).best_effort();
  bool is_custom() const { return static_cast<bool>(custom); }
};

struct ServiceRouteSpec {
  std::string ros_service;
  std::string bus_service;
  Direction direction = Direction::Ros2ToBus;
  double timeout_secs = kServiceCallTimeoutSecs;
  ServiceBuiltin builtin = ServiceBuiltin::Trigger;
  std::shared_ptr<ServiceMapper> custom;
  TopicQos ros_qos = TopicQos::keep_last(10).reliable();
  bool is_custom() const { return static_cast<bool>(custom); }
};

struct ActionRouteSpec {
  std::string ros_action;
  std::string bus_action;
  Direction direction = Direction::Ros2ToBus;
  double timeout_secs = kActionCallTimeoutSecs;
  ActionBuiltin builtin = ActionBuiltin::Fibonacci;
  std::shared_ptr<ActionMapper> custom;
  TopicQos ros_qos = TopicQos::keep_last(10).reliable();
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

/// After `from_ros`: only `to_bus`.
class Ros2BridgeFromRos {
 public:
  Ros2BridgeFromRos(std::shared_ptr<detail::BuilderState> state, std::string ros_topic,
                    TopicQos ros_qos)
      : state_(std::move(state)), ros_topic_(std::move(ros_topic)), ros_qos_(ros_qos) {}

  Ros2BridgeFromRos(const Ros2BridgeFromRos &) = delete;
  Ros2BridgeFromRos &operator=(const Ros2BridgeFromRos &) = delete;
  Ros2BridgeFromRos(Ros2BridgeFromRos &&) noexcept = default;
  Ros2BridgeFromRos &operator=(Ros2BridgeFromRos &&) noexcept = default;

  Ros2BridgeFromRosToBus to_bus(std::string bus_topic, TopicQos bus_qos) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_topic_;
  TopicQos ros_qos_;
};

/// After `to_bus`: only `mapper`.
class Ros2BridgeFromRosToBus {
 public:
  Ros2BridgeFromRosToBus(std::shared_ptr<detail::BuilderState> state, std::string ros_topic,
                         TopicQos ros_qos, std::string bus_topic, TopicQos bus_qos)
      : state_(std::move(state)),
        ros_topic_(std::move(ros_topic)),
        ros_qos_(ros_qos),
        bus_topic_(std::move(bus_topic)),
        bus_qos_(bus_qos) {}

  Ros2BridgeFromRosToBus(const Ros2BridgeFromRosToBus &) = delete;
  Ros2BridgeFromRosToBus &operator=(const Ros2BridgeFromRosToBus &) = delete;
  Ros2BridgeFromRosToBus(Ros2BridgeFromRosToBus &&) noexcept = default;
  Ros2BridgeFromRosToBus &operator=(Ros2BridgeFromRosToBus &&) noexcept = default;

  Ros2BridgeRos2ToBusReady mapper(StdMsgsStringMapper) &&;
  Ros2BridgeRos2ToBusReady mapper(SensorMsgsImageMapper) &&;
  Ros2BridgeRos2ToBusReady mapper(std::shared_ptr<TopicMapper> mapper) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_topic_;
  TopicQos ros_qos_;
  std::string bus_topic_;
  TopicQos bus_qos_;
};

/// ROS2→bus: `lazy()` / `add()`.
class Ros2BridgeRos2ToBusReady {
 public:
  Ros2BridgeRos2ToBusReady(std::shared_ptr<detail::BuilderState> state, std::string ros_topic,
                           TopicQos ros_qos, std::string bus_topic, TopicQos bus_qos,
                           detail::TopicBuiltin builtin, std::shared_ptr<TopicMapper> custom)
      : state_(std::move(state)),
        ros_topic_(std::move(ros_topic)),
        ros_qos_(ros_qos),
        bus_topic_(std::move(bus_topic)),
        bus_qos_(bus_qos),
        builtin_(builtin),
        custom_(std::move(custom)) {}

  Ros2BridgeRos2ToBusReady(const Ros2BridgeRos2ToBusReady &) = delete;
  Ros2BridgeRos2ToBusReady &operator=(const Ros2BridgeRos2ToBusReady &) = delete;
  Ros2BridgeRos2ToBusReady(Ros2BridgeRos2ToBusReady &&) noexcept = default;
  Ros2BridgeRos2ToBusReady &operator=(Ros2BridgeRos2ToBusReady &&) noexcept = default;

  Ros2BridgeRos2ToBusReady &&lazy() && {
    lazy_ = true;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_topic_;
  TopicQos ros_qos_;
  std::string bus_topic_;
  TopicQos bus_qos_;
  detail::TopicBuiltin builtin_ = detail::TopicBuiltin::StdMsgsString;
  std::shared_ptr<TopicMapper> custom_;
  bool lazy_ = false;
};

/// After `from_bus`: only `to_ros`.
class Ros2BridgeFromBus {
 public:
  Ros2BridgeFromBus(std::shared_ptr<detail::BuilderState> state, std::string bus_topic,
                    TopicQos bus_qos)
      : state_(std::move(state)), bus_topic_(std::move(bus_topic)), bus_qos_(bus_qos) {}

  Ros2BridgeFromBus(const Ros2BridgeFromBus &) = delete;
  Ros2BridgeFromBus &operator=(const Ros2BridgeFromBus &) = delete;
  Ros2BridgeFromBus(Ros2BridgeFromBus &&) noexcept = default;
  Ros2BridgeFromBus &operator=(Ros2BridgeFromBus &&) noexcept = default;

  Ros2BridgeFromBusToRos to_ros(std::string ros_topic, TopicQos ros_qos) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string bus_topic_;
  TopicQos bus_qos_;
};

class Ros2BridgeFromBusToRos {
 public:
  Ros2BridgeFromBusToRos(std::shared_ptr<detail::BuilderState> state, std::string ros_topic,
                         TopicQos ros_qos, std::string bus_topic, TopicQos bus_qos)
      : state_(std::move(state)),
        ros_topic_(std::move(ros_topic)),
        ros_qos_(ros_qos),
        bus_topic_(std::move(bus_topic)),
        bus_qos_(bus_qos) {}

  Ros2BridgeFromBusToRos(const Ros2BridgeFromBusToRos &) = delete;
  Ros2BridgeFromBusToRos &operator=(const Ros2BridgeFromBusToRos &) = delete;
  Ros2BridgeFromBusToRos(Ros2BridgeFromBusToRos &&) noexcept = default;
  Ros2BridgeFromBusToRos &operator=(Ros2BridgeFromBusToRos &&) noexcept = default;

  Ros2BridgeBusToRosReady mapper(StdMsgsStringMapper) &&;
  Ros2BridgeBusToRosReady mapper(SensorMsgsImageMapper) &&;
  Ros2BridgeBusToRosReady mapper(std::shared_ptr<TopicMapper> mapper) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_topic_;
  TopicQos ros_qos_;
  std::string bus_topic_;
  TopicQos bus_qos_;
};

class Ros2BridgeBusToRosReady {
 public:
  Ros2BridgeBusToRosReady(std::shared_ptr<detail::BuilderState> state, std::string ros_topic,
                          TopicQos ros_qos, std::string bus_topic, TopicQos bus_qos,
                          detail::TopicBuiltin builtin, std::shared_ptr<TopicMapper> custom)
      : state_(std::move(state)),
        ros_topic_(std::move(ros_topic)),
        ros_qos_(ros_qos),
        bus_topic_(std::move(bus_topic)),
        bus_qos_(bus_qos),
        builtin_(builtin),
        custom_(std::move(custom)) {}

  Ros2BridgeBusToRosReady(const Ros2BridgeBusToRosReady &) = delete;
  Ros2BridgeBusToRosReady &operator=(const Ros2BridgeBusToRosReady &) = delete;
  Ros2BridgeBusToRosReady(Ros2BridgeBusToRosReady &&) noexcept = default;
  Ros2BridgeBusToRosReady &operator=(Ros2BridgeBusToRosReady &&) noexcept = default;

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_topic_;
  TopicQos ros_qos_;
  std::string bus_topic_;
  TopicQos bus_qos_;
  detail::TopicBuiltin builtin_ = detail::TopicBuiltin::StdMsgsString;
  std::shared_ptr<TopicMapper> custom_;
};

/// After `.service()`: only `from_ros` / `from_bus`.
class Ros2BridgeService {
 public:
  explicit Ros2BridgeService(std::shared_ptr<detail::BuilderState> state)
      : state_(std::move(state)) {}

  Ros2BridgeService(const Ros2BridgeService &) = delete;
  Ros2BridgeService &operator=(const Ros2BridgeService &) = delete;
  Ros2BridgeService(Ros2BridgeService &&) noexcept = default;
  Ros2BridgeService &operator=(Ros2BridgeService &&) noexcept = default;

  Ros2BridgeServiceFromRos from_ros(std::string ros_service, TopicQos ros_qos) &&;
  Ros2BridgeServiceFromBus from_bus(std::string bus_service) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
};

class Ros2BridgeServiceFromRos {
 public:
  Ros2BridgeServiceFromRos(std::shared_ptr<detail::BuilderState> state, std::string ros_service,
                           TopicQos ros_qos)
      : state_(std::move(state)), ros_service_(std::move(ros_service)), ros_qos_(ros_qos) {}

  Ros2BridgeServiceFromRos(const Ros2BridgeServiceFromRos &) = delete;
  Ros2BridgeServiceFromRos &operator=(const Ros2BridgeServiceFromRos &) = delete;
  Ros2BridgeServiceFromRos(Ros2BridgeServiceFromRos &&) noexcept = default;
  Ros2BridgeServiceFromRos &operator=(Ros2BridgeServiceFromRos &&) noexcept = default;

  Ros2BridgeServicePair to_bus(std::string bus_service) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_service_;
  TopicQos ros_qos_;
};

class Ros2BridgeServiceFromBus {
 public:
  Ros2BridgeServiceFromBus(std::shared_ptr<detail::BuilderState> state, std::string bus_service)
      : state_(std::move(state)), bus_service_(std::move(bus_service)) {}

  Ros2BridgeServiceFromBus(const Ros2BridgeServiceFromBus &) = delete;
  Ros2BridgeServiceFromBus &operator=(const Ros2BridgeServiceFromBus &) = delete;
  Ros2BridgeServiceFromBus(Ros2BridgeServiceFromBus &&) noexcept = default;
  Ros2BridgeServiceFromBus &operator=(Ros2BridgeServiceFromBus &&) noexcept = default;

  Ros2BridgeServicePair to_ros(std::string ros_service, TopicQos ros_qos) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string bus_service_;
};

class Ros2BridgeServicePair {
 public:
  Ros2BridgeServicePair(std::shared_ptr<detail::BuilderState> state, std::string ros_service,
                        std::string bus_service, TopicQos ros_qos, Direction direction)
      : state_(std::move(state)),
        ros_service_(std::move(ros_service)),
        bus_service_(std::move(bus_service)),
        ros_qos_(ros_qos),
        direction_(direction) {}

  Ros2BridgeServicePair(const Ros2BridgeServicePair &) = delete;
  Ros2BridgeServicePair &operator=(const Ros2BridgeServicePair &) = delete;
  Ros2BridgeServicePair(Ros2BridgeServicePair &&) noexcept = default;
  Ros2BridgeServicePair &operator=(Ros2BridgeServicePair &&) noexcept = default;

  Ros2BridgeServiceReady mapper(TriggerServiceMapper) &&;
  Ros2BridgeServiceReady mapper(SetBoolServiceMapper) &&;
  Ros2BridgeServiceReady mapper(std::shared_ptr<ServiceMapper> mapper) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_service_;
  std::string bus_service_;
  TopicQos ros_qos_;
  Direction direction_;
};

class Ros2BridgeServiceReady {
 public:
  Ros2BridgeServiceReady(std::shared_ptr<detail::BuilderState> state, std::string ros_service,
                         std::string bus_service, Direction direction, TopicQos ros_qos,
                         detail::ServiceBuiltin builtin, std::shared_ptr<ServiceMapper> custom)
      : state_(std::move(state)),
        ros_service_(std::move(ros_service)),
        bus_service_(std::move(bus_service)),
        direction_(direction),
        ros_qos_(ros_qos),
        builtin_(builtin),
        custom_(std::move(custom)) {}

  Ros2BridgeServiceReady(const Ros2BridgeServiceReady &) = delete;
  Ros2BridgeServiceReady &operator=(const Ros2BridgeServiceReady &) = delete;
  Ros2BridgeServiceReady(Ros2BridgeServiceReady &&) noexcept = default;
  Ros2BridgeServiceReady &operator=(Ros2BridgeServiceReady &&) noexcept = default;

  Ros2BridgeServiceReady &&timeout(double timeout_secs) && {
    timeout_secs_ = timeout_secs;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_service_;
  std::string bus_service_;
  Direction direction_;
  TopicQos ros_qos_ = TopicQos::keep_last(10).reliable();
  double timeout_secs_ = kServiceCallTimeoutSecs;
  detail::ServiceBuiltin builtin_ = detail::ServiceBuiltin::Trigger;
  std::shared_ptr<ServiceMapper> custom_;
};

/// After `.action()`: only `from_ros` / `from_bus`.
class Ros2BridgeAction {
 public:
  explicit Ros2BridgeAction(std::shared_ptr<detail::BuilderState> state)
      : state_(std::move(state)) {}

  Ros2BridgeAction(const Ros2BridgeAction &) = delete;
  Ros2BridgeAction &operator=(const Ros2BridgeAction &) = delete;
  Ros2BridgeAction(Ros2BridgeAction &&) noexcept = default;
  Ros2BridgeAction &operator=(Ros2BridgeAction &&) noexcept = default;

  Ros2BridgeActionFromRos from_ros(std::string ros_action, TopicQos ros_qos) &&;
  Ros2BridgeActionFromBus from_bus(std::string bus_action) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
};

class Ros2BridgeActionFromRos {
 public:
  Ros2BridgeActionFromRos(std::shared_ptr<detail::BuilderState> state, std::string ros_action,
                          TopicQos ros_qos)
      : state_(std::move(state)), ros_action_(std::move(ros_action)), ros_qos_(ros_qos) {}

  Ros2BridgeActionFromRos(const Ros2BridgeActionFromRos &) = delete;
  Ros2BridgeActionFromRos &operator=(const Ros2BridgeActionFromRos &) = delete;
  Ros2BridgeActionFromRos(Ros2BridgeActionFromRos &&) noexcept = default;
  Ros2BridgeActionFromRos &operator=(Ros2BridgeActionFromRos &&) noexcept = default;

  Ros2BridgeActionPair to_bus(std::string bus_action) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_action_;
  TopicQos ros_qos_;
};

class Ros2BridgeActionFromBus {
 public:
  Ros2BridgeActionFromBus(std::shared_ptr<detail::BuilderState> state, std::string bus_action)
      : state_(std::move(state)), bus_action_(std::move(bus_action)) {}

  Ros2BridgeActionFromBus(const Ros2BridgeActionFromBus &) = delete;
  Ros2BridgeActionFromBus &operator=(const Ros2BridgeActionFromBus &) = delete;
  Ros2BridgeActionFromBus(Ros2BridgeActionFromBus &&) noexcept = default;
  Ros2BridgeActionFromBus &operator=(Ros2BridgeActionFromBus &&) noexcept = default;

  Ros2BridgeActionPair to_ros(std::string ros_action, TopicQos ros_qos) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string bus_action_;
};

class Ros2BridgeActionPair {
 public:
  Ros2BridgeActionPair(std::shared_ptr<detail::BuilderState> state, std::string ros_action,
                       std::string bus_action, TopicQos ros_qos, Direction direction)
      : state_(std::move(state)),
        ros_action_(std::move(ros_action)),
        bus_action_(std::move(bus_action)),
        ros_qos_(ros_qos),
        direction_(direction) {}

  Ros2BridgeActionPair(const Ros2BridgeActionPair &) = delete;
  Ros2BridgeActionPair &operator=(const Ros2BridgeActionPair &) = delete;
  Ros2BridgeActionPair(Ros2BridgeActionPair &&) noexcept = default;
  Ros2BridgeActionPair &operator=(Ros2BridgeActionPair &&) noexcept = default;

  Ros2BridgeActionReady mapper(FibonacciActionMapper) &&;
  Ros2BridgeActionReady mapper(std::shared_ptr<ActionMapper> mapper) &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_action_;
  std::string bus_action_;
  TopicQos ros_qos_;
  Direction direction_;
};

class Ros2BridgeActionReady {
 public:
  Ros2BridgeActionReady(std::shared_ptr<detail::BuilderState> state, std::string ros_action,
                        std::string bus_action, Direction direction, TopicQos ros_qos,
                        detail::ActionBuiltin builtin, std::shared_ptr<ActionMapper> custom)
      : state_(std::move(state)),
        ros_action_(std::move(ros_action)),
        bus_action_(std::move(bus_action)),
        direction_(direction),
        ros_qos_(ros_qos),
        builtin_(builtin),
        custom_(std::move(custom)) {}

  Ros2BridgeActionReady(const Ros2BridgeActionReady &) = delete;
  Ros2BridgeActionReady &operator=(const Ros2BridgeActionReady &) = delete;
  Ros2BridgeActionReady(Ros2BridgeActionReady &&) noexcept = default;
  Ros2BridgeActionReady &operator=(Ros2BridgeActionReady &&) noexcept = default;

  Ros2BridgeActionReady &&timeout(double timeout_secs) && {
    timeout_secs_ = timeout_secs;
    return std::move(*this);
  }

  Ros2BridgeBuilder add() &&;

 private:
  std::shared_ptr<detail::BuilderState> state_;
  std::string ros_action_;
  std::string bus_action_;
  Direction direction_;
  TopicQos ros_qos_ = TopicQos::keep_last(10).reliable();
  double timeout_secs_ = kActionCallTimeoutSecs;
  detail::ActionBuiltin builtin_ = detail::ActionBuiltin::Fibonacci;
  std::shared_ptr<ActionMapper> custom_;
};

/// Fluent builder: `Ros2Bridge::New(name).from_ros(...).to_bus(...).mapper(...).add().build()`.
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

  Ros2BridgeFromRos from_ros(std::string ros_topic, TopicQos ros_qos) && {
    auto state = std::move(state_);
    return Ros2BridgeFromRos(std::move(state), std::move(ros_topic), ros_qos);
  }

  Ros2BridgeFromBus from_bus(std::string bus_topic, TopicQos bus_qos) && {
    require_bus_best_effort(bus_qos);
    auto state = std::move(state_);
    return Ros2BridgeFromBus(std::move(state), std::move(bus_topic), bus_qos);
  }

  Ros2BridgeService service() && {
    auto state = std::move(state_);
    return Ros2BridgeService(std::move(state));
  }

  Ros2BridgeAction action() && {
    auto state = std::move(state_);
    return Ros2BridgeAction(std::move(state));
  }

  Ros2Bridge build() &&;

 private:
  friend class Ros2BridgeFromRos;
  friend class Ros2BridgeFromRosToBus;
  friend class Ros2BridgeRos2ToBusReady;
  friend class Ros2BridgeFromBus;
  friend class Ros2BridgeFromBusToRos;
  friend class Ros2BridgeBusToRosReady;
  friend class Ros2BridgeService;
  friend class Ros2BridgeServiceFromRos;
  friend class Ros2BridgeServiceFromBus;
  friend class Ros2BridgeServicePair;
  friend class Ros2BridgeServiceReady;
  friend class Ros2BridgeAction;
  friend class Ros2BridgeActionFromRos;
  friend class Ros2BridgeActionFromBus;
  friend class Ros2BridgeActionPair;
  friend class Ros2BridgeActionReady;
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

inline Ros2BridgeFromRosToBus Ros2BridgeFromRos::to_bus(std::string bus_topic,
                                                       TopicQos bus_qos) && {
  require_bus_best_effort(bus_qos);
  return Ros2BridgeFromRosToBus(std::move(state_), std::move(ros_topic_), ros_qos_,
                                std::move(bus_topic), bus_qos);
}

inline Ros2BridgeRos2ToBusReady Ros2BridgeFromRosToBus::mapper(StdMsgsStringMapper) && {
  return Ros2BridgeRos2ToBusReady(std::move(state_), std::move(ros_topic_), ros_qos_,
                                  std::move(bus_topic_), bus_qos_,
                                  detail::TopicBuiltin::StdMsgsString, nullptr);
}

inline Ros2BridgeRos2ToBusReady Ros2BridgeFromRosToBus::mapper(SensorMsgsImageMapper) && {
  return Ros2BridgeRos2ToBusReady(std::move(state_), std::move(ros_topic_), ros_qos_,
                                  std::move(bus_topic_), bus_qos_,
                                  detail::TopicBuiltin::SensorMsgsImage, nullptr);
}

inline Ros2BridgeRos2ToBusReady Ros2BridgeFromRosToBus::mapper(
    std::shared_ptr<TopicMapper> mapper) && {
  if (!mapper) {
    throw Error("ros2 bridge route: mapper shared_ptr must not be null");
  }
  return Ros2BridgeRos2ToBusReady(std::move(state_), std::move(ros_topic_), ros_qos_,
                                  std::move(bus_topic_), bus_qos_,
                                  detail::TopicBuiltin::StdMsgsString, std::move(mapper));
}

inline Ros2BridgeBuilder Ros2BridgeRos2ToBusReady::add() && {
  if (lazy_ && custom_ && !custom_->supports_lazy()) {
    throw Error(
        "ros2 bridge route: .lazy() is not supported for this custom TopicMapper "
        "(attach-only); use TypedTopicMapper");
  }
  detail::TopicRouteSpec spec;
  spec.ros_topic = std::move(ros_topic_);
  spec.bus_topic = std::move(bus_topic_);
  spec.direction = Direction::Ros2ToBus;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  spec.lazy = lazy_;
  spec.ros_qos = ros_qos_;
  spec.bus_qos = bus_qos_;
  state_->routes.push_back(std::move(spec));
  return Ros2BridgeBuilder(std::move(state_));
}

inline Ros2BridgeFromBusToRos Ros2BridgeFromBus::to_ros(std::string ros_topic,
                                                       TopicQos ros_qos) && {
  return Ros2BridgeFromBusToRos(std::move(state_), std::move(ros_topic), ros_qos,
                                std::move(bus_topic_), bus_qos_);
}

inline Ros2BridgeBusToRosReady Ros2BridgeFromBusToRos::mapper(StdMsgsStringMapper) && {
  return Ros2BridgeBusToRosReady(std::move(state_), std::move(ros_topic_), ros_qos_,
                                 std::move(bus_topic_), bus_qos_,
                                 detail::TopicBuiltin::StdMsgsString, nullptr);
}

inline Ros2BridgeBusToRosReady Ros2BridgeFromBusToRos::mapper(SensorMsgsImageMapper) && {
  return Ros2BridgeBusToRosReady(std::move(state_), std::move(ros_topic_), ros_qos_,
                                 std::move(bus_topic_), bus_qos_,
                                 detail::TopicBuiltin::SensorMsgsImage, nullptr);
}

inline Ros2BridgeBusToRosReady Ros2BridgeFromBusToRos::mapper(
    std::shared_ptr<TopicMapper> mapper) && {
  if (!mapper) {
    throw Error("ros2 bridge route: mapper shared_ptr must not be null");
  }
  return Ros2BridgeBusToRosReady(std::move(state_), std::move(ros_topic_), ros_qos_,
                                 std::move(bus_topic_), bus_qos_,
                                 detail::TopicBuiltin::StdMsgsString, std::move(mapper));
}

inline Ros2BridgeBuilder Ros2BridgeBusToRosReady::add() && {
  detail::TopicRouteSpec spec;
  spec.ros_topic = std::move(ros_topic_);
  spec.bus_topic = std::move(bus_topic_);
  spec.direction = Direction::BusToRos2;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  spec.lazy = false;
  spec.ros_qos = ros_qos_;
  spec.bus_qos = bus_qos_;
  state_->routes.push_back(std::move(spec));
  return Ros2BridgeBuilder(std::move(state_));
}

inline Ros2BridgeServiceFromRos Ros2BridgeService::from_ros(std::string ros_service,
                                                            TopicQos ros_qos) && {
  return Ros2BridgeServiceFromRos(std::move(state_), std::move(ros_service), ros_qos);
}

inline Ros2BridgeServiceFromBus Ros2BridgeService::from_bus(std::string bus_service) && {
  return Ros2BridgeServiceFromBus(std::move(state_), std::move(bus_service));
}

inline Ros2BridgeServicePair Ros2BridgeServiceFromRos::to_bus(std::string bus_service) && {
  return Ros2BridgeServicePair(std::move(state_), std::move(ros_service_), std::move(bus_service),
                               ros_qos_, Direction::Ros2ToBus);
}

inline Ros2BridgeServicePair Ros2BridgeServiceFromBus::to_ros(std::string ros_service,
                                                             TopicQos ros_qos) && {
  return Ros2BridgeServicePair(std::move(state_), std::move(ros_service), std::move(bus_service_),
                               ros_qos, Direction::BusToRos2);
}

inline Ros2BridgeServiceReady Ros2BridgeServicePair::mapper(TriggerServiceMapper) && {
  return Ros2BridgeServiceReady(std::move(state_), std::move(ros_service_),
                                std::move(bus_service_), direction_, ros_qos_,
                                detail::ServiceBuiltin::Trigger, nullptr);
}

inline Ros2BridgeServiceReady Ros2BridgeServicePair::mapper(SetBoolServiceMapper) && {
  return Ros2BridgeServiceReady(std::move(state_), std::move(ros_service_),
                                std::move(bus_service_), direction_, ros_qos_,
                                detail::ServiceBuiltin::SetBool, nullptr);
}

inline Ros2BridgeServiceReady Ros2BridgeServicePair::mapper(
    std::shared_ptr<ServiceMapper> mapper) && {
  if (!mapper) {
    throw Error("ros2 bridge service: mapper shared_ptr must not be null");
  }
  return Ros2BridgeServiceReady(std::move(state_), std::move(ros_service_),
                                std::move(bus_service_), direction_, ros_qos_,
                                detail::ServiceBuiltin::Trigger, std::move(mapper));
}

inline Ros2BridgeBuilder Ros2BridgeServiceReady::add() && {
  detail::ServiceRouteSpec spec;
  spec.ros_service = std::move(ros_service_);
  spec.bus_service = std::move(bus_service_);
  spec.direction = direction_;
  spec.timeout_secs = timeout_secs_;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  spec.ros_qos = ros_qos_;
  state_->services.push_back(std::move(spec));
  return Ros2BridgeBuilder(std::move(state_));
}

inline Ros2BridgeActionFromRos Ros2BridgeAction::from_ros(std::string ros_action,
                                                         TopicQos ros_qos) && {
  return Ros2BridgeActionFromRos(std::move(state_), std::move(ros_action), ros_qos);
}

inline Ros2BridgeActionFromBus Ros2BridgeAction::from_bus(std::string bus_action) && {
  return Ros2BridgeActionFromBus(std::move(state_), std::move(bus_action));
}

inline Ros2BridgeActionPair Ros2BridgeActionFromRos::to_bus(std::string bus_action) && {
  return Ros2BridgeActionPair(std::move(state_), std::move(ros_action_), std::move(bus_action),
                              ros_qos_, Direction::Ros2ToBus);
}

inline Ros2BridgeActionPair Ros2BridgeActionFromBus::to_ros(std::string ros_action,
                                                           TopicQos ros_qos) && {
  return Ros2BridgeActionPair(std::move(state_), std::move(ros_action), std::move(bus_action_),
                              ros_qos, Direction::BusToRos2);
}

inline Ros2BridgeActionReady Ros2BridgeActionPair::mapper(FibonacciActionMapper) && {
  return Ros2BridgeActionReady(std::move(state_), std::move(ros_action_), std::move(bus_action_),
                               direction_, ros_qos_, detail::ActionBuiltin::Fibonacci, nullptr);
}

inline Ros2BridgeActionReady Ros2BridgeActionPair::mapper(
    std::shared_ptr<ActionMapper> mapper) && {
  if (!mapper) {
    throw Error("ros2 bridge action: mapper shared_ptr must not be null");
  }
  return Ros2BridgeActionReady(std::move(state_), std::move(ros_action_), std::move(bus_action_),
                               direction_, ros_qos_, detail::ActionBuiltin::Fibonacci,
                               std::move(mapper));
}

inline Ros2BridgeBuilder Ros2BridgeActionReady::add() && {
  detail::ActionRouteSpec spec;
  spec.ros_action = std::move(ros_action_);
  spec.bus_action = std::move(bus_action_);
  spec.direction = direction_;
  spec.timeout_secs = timeout_secs_;
  spec.builtin = builtin_;
  spec.custom = std::move(custom_);
  spec.ros_qos = ros_qos_;
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
