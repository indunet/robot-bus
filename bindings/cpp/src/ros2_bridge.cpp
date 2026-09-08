#include "ros2_bridge_internal.hpp"

#include <robot_bus/robot_bus_interfaces/msg/v1/console_status.pb.h>

#include <rclcpp/rclcpp.hpp>
#include <rclcpp_action/rclcpp_action.hpp>

#include <atomic>
#include <chrono>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <thread>
#include <unordered_map>
#include <unordered_set>
#include <utility>
#include <vector>

namespace robot_bus {
using ros2_bridge_detail::ConsoleRoute;
using ros2_bridge_detail::LazyTopic;
using ros2_bridge_detail::action_type_name;
using ros2_bridge_detail::kBridges;
using ros2_bridge_detail::kConsoleDetectTimeoutSecs;
using ros2_bridge_detail::kEvents;
using ros2_bridge_detail::kIdleGraceSecs;
using ros2_bridge_detail::kTopicDemand;
using ros2_bridge_detail::kTopicsSnapshot;
using ros2_bridge_detail::log_route_table;
using ros2_bridge_detail::make_bus_node;
using ros2_bridge_detail::make_topic_console_route;
using ros2_bridge_detail::service_type_name;
using ros2_bridge_detail::should_enable_ros_subscription;
using ros2_bridge_detail::wire_action;
using ros2_bridge_detail::wire_service;
using ros2_bridge_detail::wire_topic;

void TopicMapper::attach(TopicWireContext &ctx) {
  (void)ctx;
  throw Error("custom TopicMapper must override attach()");
}

rclcpp::SubscriptionBase::SharedPtr TopicMapper::create_ros2_to_bus_subscription(
    rclcpp::Node::SharedPtr, const std::string &, std::shared_ptr<TopicPublisher>,
    std::shared_ptr<std::mutex>, const rclcpp::QoS &, std::shared_ptr<DropStats>,
    std::shared_ptr<RouteHealth>) {
  throw Error("custom TopicMapper does not support .lazy()");
}

void ServiceMapper::attach(ServiceWireContext &ctx) {
  (void)ctx;
  throw Error(std::string("custom ServiceMapper must override attach(); type=") + type_name());
}

void ActionMapper::attach(ActionWireContext &ctx) {
  (void)ctx;
  throw Error(std::string("custom ActionMapper must override attach(); type=") + type_name());
}

struct Ros2Bridge::Impl {
  Node bus_node;
  rclcpp::Node::SharedPtr ros_node;
  rclcpp::executors::MultiThreadedExecutor::SharedPtr executor;
  rclcpp::CallbackGroup::SharedPtr callback_group;
  std::vector<rclcpp::SubscriptionBase::SharedPtr> ros_subs;
  std::vector<rclcpp::PublisherBase::SharedPtr> ros_pubs;
  std::vector<rclcpp::ServiceBase::SharedPtr> ros_srvs;
  std::vector<rclcpp::ClientBase::SharedPtr> ros_clients;
  std::vector<std::shared_ptr<rclcpp_action::ServerBase>> ros_actions;
  std::vector<std::shared_ptr<rclcpp_action::ClientBase>> ros_action_clients;
  std::vector<std::shared_ptr<TopicPublisher>> bus_pubs;
  std::vector<std::shared_ptr<std::mutex>> bus_pub_mutexes;
  std::vector<std::shared_ptr<ServiceClient>> bus_clients;
  std::vector<std::shared_ptr<ActionClient>> bus_action_clients;
  /// Custom mapper entities (`TopicMapper::attach` / service / action).
  std::vector<std::shared_ptr<void>> keep_alive;
  std::vector<LazyTopic> lazy_routes;
  std::unordered_set<std::string> eager_bus_topics;
  std::unordered_map<std::string, uint32_t> subscriber_counts;
  std::optional<bool> console_live;
  std::optional<std::chrono::steady_clock::time_point> first_spin;
  std::atomic<bool> halt{false};
  std::thread spin_thread;
  std::shared_ptr<DropStats> drop_stats = std::make_shared<DropStats>();
  std::vector<ConsoleRoute> console_routes;
  std::string bridge_id;
  std::string bridge_name;
  std::shared_ptr<TopicPublisher> bridges_pub;
  std::shared_ptr<TopicPublisher> events_pub;
  std::optional<std::chrono::steady_clock::time_point> last_snapshot;
  uint64_t event_seq = 0;

  explicit Impl(Node bus) : bus_node(std::move(bus)) {}

  ~Impl() {
    halt.store(true, std::memory_order_relaxed);
    if (executor) {
      executor->cancel();
    }
    if (spin_thread.joinable()) {
      spin_thread.join();
    }
    if (executor && ros_node) {
      executor->remove_node(ros_node);
    }
  }

  void apply_lazy() {
    if (lazy_routes.empty()) {
      return;
    }
    if (!console_live.has_value() && first_spin.has_value()) {
      const auto elapsed = std::chrono::steady_clock::now() - *first_spin;
      if (elapsed >= std::chrono::duration<double>(kConsoleDetectTimeoutSecs)) {
        console_live = false;
      }
    }
    for (auto &route : lazy_routes) {
      const uint32_t n =
          subscriber_counts.count(route.bus_topic) ? subscriber_counts[route.bus_topic] : 0;
      const bool want = should_enable_ros_subscription(true, console_live, n);
      if (want && !route.sub) {
        try {
          route.sub = route.create();
        } catch (const std::exception &e) {
          (void)e;
        }
      } else if (!want && route.sub) {
        route.sub.reset();
      }
    }
  }

  void subscribe_demand() {
    auto *self = this;
    keep_alive.push_back(std::make_shared<SubscriptionHandle>(bus_node.create_subscription(
        kTopicDemand, [self](BytesView payload) {
          robot_bus_interfaces::msg::v1::TopicDemand msg;
          if (!msg.ParseFromArray(payload.data, static_cast<int>(payload.size))) {
            return;
          }
          self->console_live = true;
          self->subscriber_counts[msg.topic()] = msg.subscribers();
        })));
    keep_alive.push_back(std::make_shared<SubscriptionHandle>(bus_node.create_subscription(
        kTopicsSnapshot, [self](BytesView payload) {
          robot_bus_interfaces::msg::v1::TopicStatsList list;
          if (!list.ParseFromArray(payload.data, static_cast<int>(payload.size))) {
            return;
          }
          self->console_live = true;
          for (const auto &t : list.topics()) {
            self->subscriber_counts[t.name()] = static_cast<uint32_t>(t.subscribers());
          }
        })));
  }

  bool route_enabled(const ConsoleRoute &route) const {
    if (!route.lazy) {
      return true;
    }
    for (const auto &lazy : lazy_routes) {
      if (lazy.bus_topic == route.bus_name) {
        return static_cast<bool>(lazy.sub);
      }
    }
    return eager_bus_topics.count(route.bus_name) != 0;
  }

  bool grace_elapsed() const {
    if (!first_spin.has_value()) {
      return false;
    }
    return (std::chrono::steady_clock::now() - *first_spin) >=
           std::chrono::duration<double>(kIdleGraceSecs);
  }

  void publish_observe() {
    const auto now = std::chrono::steady_clock::now();
    if (last_snapshot.has_value() &&
        (now - *last_snapshot) < std::chrono::seconds(1)) {
      return;
    }
    last_snapshot = now;
    if (!bridges_pub) {
      return;
    }
    const bool grace = grace_elapsed();
    robot_bus_interfaces::msg::v1::BridgeSnapshot snap;
    snap.set_bridge_id(bridge_id);
    snap.set_bridge_name(bridge_name);
    for (const auto &route : console_routes) {
      const bool enabled = route_enabled(route);
      const auto health = route.health;
      auto *proto = snap.add_routes();
      proto->set_kind(route.kind);
      proto->set_direction(route.direction);
      proto->set_ros_name(route.ros_name);
      proto->set_bus_name(route.bus_name);
      proto->set_type_name(route.type_name);
      proto->set_ros_qos(route.ros_qos);
      proto->set_bus_qos(route.bus_qos);
      proto->set_lazy(route.lazy);
      proto->set_enabled(enabled);
      if (health) {
        proto->set_rx(health->rx.load(std::memory_order_relaxed));
        proto->set_tx(health->tx.load(std::memory_order_relaxed));
        proto->set_convert_fail(health->convert_fail.load(std::memory_order_relaxed));
        proto->set_decode_fail(health->decode_fail.load(std::memory_order_relaxed));
        proto->set_publish_fail(health->publish_fail.load(std::memory_order_relaxed));
        proto->set_last_rx_ms(health->last_rx_ms.load(std::memory_order_relaxed));
        proto->set_idle(route.watch_idle && health->is_idle(enabled, grace));
        const auto rpc = health->rpc_snapshot();
        proto->set_calls(rpc.calls); proto->set_failures(rpc.failures);
        proto->set_timeouts(rpc.timeouts); proto->set_cancelled(rpc.cancelled);
        proto->set_rejected(rpc.rejected); proto->set_last_error(rpc.last_error);
        proto->set_last_status(rpc.last_status);
      }
    }
    const auto bytes = snap.SerializeAsString();
    try {
      bridges_pub->publish(std::vector<uint8_t>(bytes.begin(), bytes.end()));
    } catch (...) {
    }
    for (const auto &route : console_routes) {
      if (!route.watch_idle || !route.health) {
        continue;
      }
      const bool enabled = route_enabled(route);
      if (!route.health->take_idle_event(enabled, grace)) {
        continue;
      }
      const std::string msg =
          "no traffic on " + route.direction + " " + route.ros_name +
          " for 15s; check source traffic, connection, direction and ROS QoS";
      RCLCPP_WARN(ros2_bridge_logger(), "ros2_bridge/%s: %s", bridge_name.c_str(), msg.c_str());
      if (!events_pub) {
        continue;
      }
      ++event_seq;
      robot_bus_interfaces::msg::v1::ConsoleEvent ev;
      ev.set_id("bridge-idle-" + std::to_string(event_seq));
      ev.set_ts(RouteHealth::unix_ms());
      ev.set_level("WARN");
      ev.set_source("ros2_bridge/" + bridge_name);
      ev.set_message(msg);
      const auto ev_bytes = ev.SerializeAsString();
      try {
        events_pub->publish(std::vector<uint8_t>(ev_bytes.begin(), ev_bytes.end()));
      } catch (...) {
      }
    }
  }
};

Ros2Bridge::Ros2Bridge(std::unique_ptr<Impl> impl) : impl_(std::move(impl)) {}

Ros2Bridge::~Ros2Bridge() = default;

Ros2Bridge::Ros2Bridge(Ros2Bridge &&) noexcept = default;

Ros2Bridge &Ros2Bridge::operator=(Ros2Bridge &&) noexcept = default;

void Ros2Bridge::spin() {
  while (true) {
    spin_once(-1.0);
  }
}

void Ros2Bridge::spin_once(double timeout_secs) {
  if (!impl_) {
    throw Error("Ros2Bridge is empty");
  }
  if (!impl_->first_spin.has_value()) {
    impl_->first_spin = std::chrono::steady_clock::now();
  }
  try {
    impl_->bus_node.spin_once(timeout_secs);
  } catch (const Error &e) {
    // Ros2ToBus-only may leave the bus node with no sub/service/action server.
    const std::string msg = e.what();
    if (msg.find("nothing registered") == std::string::npos) {
      throw;
    }
  }
  impl_->apply_lazy();
  impl_->publish_observe();
}

bool Ros2Bridge::has_ros_subscription(const std::string &bus_topic) const {
  if (!impl_) {
    return false;
  }
  for (const auto &route : impl_->lazy_routes) {
    if (route.bus_topic == bus_topic) {
      return static_cast<bool>(route.sub);
    }
  }
  return impl_->eager_bus_topics.count(bus_topic) != 0;
}

DropStatsSnapshot Ros2Bridge::drop_stats() const {
  if (!impl_ || !impl_->drop_stats) {
    return {};
  }
  return impl_->drop_stats->snapshot();
}

Ros2Bridge Ros2BridgeBuilder::build() && {
  if (!state_) {
    throw Error("Ros2Bridge builder already consumed");
  }
  auto state = std::move(state_);
  if (state->routes.empty() && state->services.empty() && state->actions.empty()) {
    throw Error("Ros2Bridge requires at least one topic route, service, or action");
  }

  if (!rclcpp::ok()) {
    rclcpp::init(0, nullptr);
  }

  auto impl = std::make_unique<Ros2Bridge::Impl>(make_bus_node(*state));
  impl->ros_node = std::make_shared<rclcpp::Node>(state->name);
  impl->callback_group =
      impl->ros_node->create_callback_group(rclcpp::CallbackGroupType::Reentrant);
  impl->executor = std::make_shared<rclcpp::executors::MultiThreadedExecutor>();
  impl->executor->add_node(impl->ros_node);

  for (const auto &route : state->routes) {
    auto health = std::make_shared<RouteHealth>();
    health->latched = route.ros_qos.is_transient_local();
    wire_topic(impl->ros_node, impl->bus_node, route, impl->ros_subs, impl->ros_pubs,
               impl->bus_pubs, impl->bus_pub_mutexes, impl->keep_alive, impl->lazy_routes,
               impl->eager_bus_topics, impl->drop_stats, health);
    impl->console_routes.push_back(make_topic_console_route(route, health));
  }
  for (auto svc : state->services) {
    svc.health = std::make_shared<RouteHealth>();
    wire_service(impl->ros_node, impl->bus_node, svc, impl->callback_group, impl->ros_srvs,
                 impl->ros_clients, impl->bus_clients, impl->keep_alive);
    impl->console_routes.push_back(ConsoleRoute{
        "service",
        direction_label(svc.direction),
        svc.ros_service,
        svc.bus_service,
        service_type_name(svc),
        qos_console_label(svc.ros_qos),
        qos_console_label(svc.bus_qos),
        false,
        false,
        svc.health});
  }
  for (auto act : state->actions) {
    act.health = std::make_shared<RouteHealth>();
    wire_action(impl->ros_node, impl->bus_node, act, impl->callback_group, impl->ros_actions,
                impl->ros_action_clients, impl->bus_action_clients, impl->keep_alive);
    impl->console_routes.push_back(ConsoleRoute{
        "action",
        direction_label(act.direction),
        act.ros_action,
        act.bus_action,
        action_type_name(act),
        qos_console_label(act.ros_qos),
        qos_console_label(act.bus_qos),
        false,
        false,
        act.health});
  }

  log_route_table(state->name, impl->console_routes);
  impl->bridge_name = state->name;
  impl->bridge_id = state->name + "-" + std::to_string(RouteHealth::unix_ms());
  impl->bridges_pub = std::make_shared<TopicPublisher>(impl->bus_node.create_publisher(kBridges));
  impl->events_pub = std::make_shared<TopicPublisher>(impl->bus_node.create_publisher(kEvents));

  if (!impl->lazy_routes.empty()) {
    impl->subscribe_demand();
  }

  auto *raw = impl.get();
  raw->spin_thread = std::thread([raw]() {
    // Humble MultiThreadedExecutor::spin() often ignores cancel() from another
    // thread. Timed spin_once lets ~Impl join after halt is set.
    while (!raw->halt.load(std::memory_order_relaxed)) {
      raw->executor->spin_once(std::chrono::milliseconds(50));
    }
  });

  return Ros2Bridge(std::move(impl));
}

}  // namespace robot_bus
