#include "ros2_bridge_internal.hpp"

#include <robot_bus/typed.hpp>

#include <robot_bus/builtin_interfaces/msg/v1/time.pb.h>
#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>
#include <robot_bus/sensor_msgs/msg/v1/image.pb.h>
#include <robot_bus/std_msgs/msg/v1/header.pb.h>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>
#include <robot_bus/std_srvs/srv/v1/trigger.pb.h>

#include <example_interfaces/action/fibonacci.hpp>
#include <sensor_msgs/msg/image.hpp>
#include <std_msgs/msg/string.hpp>
#include <std_srvs/srv/set_bool.hpp>
#include <std_srvs/srv/trigger.hpp>

#include <cstdint>
#include <functional>
#include <mutex>
#include <optional>
#include <string>
#include <unordered_set>
#include <utility>
#include <vector>

namespace robot_bus {
namespace ros2_bridge_detail {


using detail::BusTransportKind;
using detail::ServiceBuiltin;
using detail::TopicBuiltin;

using robot_bus::encode_pb;

template <typename T>
T parse_pb(const uint8_t *data, size_t len) {
  T msg;
  if (!msg.ParseFromArray(data, static_cast<int>(len))) {
    throw BridgeDecodeError("protobuf ParseFromArray failed");
  }
  return msg;
}

template <typename T>
T parse_pb(BytesView view) {
  return parse_pb<T>(view.data, view.size);
}

std::vector<uint8_t> string_ros_to_bus(const std_msgs::msg::String &ros) {
  std_msgs::msg::v1::String bus;
  bus.set_data(ros.data);
  return encode_pb(bus);
}

std_msgs::msg::String string_bus_to_ros(const uint8_t *data, size_t len) {
  auto bus = parse_pb<std_msgs::msg::v1::String>(data, len);
  std_msgs::msg::String ros;
  ros.data = bus.data();
  return ros;
}

std::vector<uint8_t> image_ros_to_bus(const sensor_msgs::msg::Image &ros) {
  sensor_msgs::msg::v1::Image bus;
  auto *header = bus.mutable_header();
  header->set_frame_id(ros.header.frame_id);
  auto *stamp = header->mutable_stamp();
  stamp->set_sec(ros.header.stamp.sec);
  stamp->set_nanosec(ros.header.stamp.nanosec);
  bus.set_height(ros.height);
  bus.set_width(ros.width);
  bus.set_encoding(ros.encoding);
  bus.set_is_bigendian(ros.is_bigendian != 0);
  bus.set_step(ros.step);
  bus.set_data(ros.data.data(), ros.data.size());
  return encode_pb(bus);
}

sensor_msgs::msg::Image image_bus_to_ros(const uint8_t *data, size_t len) {
  auto bus = parse_pb<sensor_msgs::msg::v1::Image>(data, len);
  sensor_msgs::msg::Image ros;
  if (bus.has_header()) {
    ros.header.frame_id = bus.header().frame_id();
    if (bus.header().has_stamp()) {
      ros.header.stamp.sec = bus.header().stamp().sec();
      ros.header.stamp.nanosec = bus.header().stamp().nanosec();
    }
  }
  ros.height = bus.height();
  ros.width = bus.width();
  ros.encoding = bus.encoding();
  ros.is_bigendian = bus.is_bigendian() ? 1 : 0;
  ros.step = bus.step();
  ros.data.assign(bus.data().begin(), bus.data().end());
  return ros;
}

std::vector<uint8_t> trigger_req_to_bus() {
  return encode_pb(std_srvs::srv::v1::TriggerRequest{});
}

std::vector<uint8_t> trigger_resp_ros_to_bus(const std_srvs::srv::Trigger::Response &ros) {
  std_srvs::srv::v1::TriggerResponse bus;
  bus.set_success(ros.success);
  bus.set_message(ros.message);
  return encode_pb(bus);
}

std_srvs::srv::Trigger::Response trigger_resp_bus_to_ros(BytesView body) {
  auto bus = parse_pb<std_srvs::srv::v1::TriggerResponse>(body);
  std_srvs::srv::Trigger::Response ros;
  ros.success = bus.success();
  ros.message = bus.message();
  return ros;
}

std::vector<uint8_t> set_bool_req_ros_to_bus(const std_srvs::srv::SetBool::Request &ros) {
  std_srvs::srv::v1::SetBoolRequest bus;
  bus.set_data(ros.data);
  return encode_pb(bus);
}

std_srvs::srv::SetBool::Request set_bool_req_bus_to_ros(BytesView body) {
  auto bus = parse_pb<std_srvs::srv::v1::SetBoolRequest>(body);
  std_srvs::srv::SetBool::Request ros;
  ros.data = bus.data();
  return ros;
}

std::vector<uint8_t> set_bool_resp_ros_to_bus(const std_srvs::srv::SetBool::Response &ros) {
  std_srvs::srv::v1::SetBoolResponse bus;
  bus.set_success(ros.success);
  bus.set_message(ros.message);
  return encode_pb(bus);
}

std_srvs::srv::SetBool::Response set_bool_resp_bus_to_ros(BytesView body) {
  auto bus = parse_pb<std_srvs::srv::v1::SetBoolResponse>(body);
  std_srvs::srv::SetBool::Response ros;
  ros.success = bus.success();
  ros.message = bus.message();
  return ros;
}

std::vector<uint8_t> fibonacci_goal_ros_to_bus(
    const example_interfaces::action::Fibonacci::Goal &ros) {
  example_interfaces::action::v1::FibonacciGoal bus;
  bus.set_order(ros.order);
  return encode_pb(bus);
}

example_interfaces::action::Fibonacci::Goal fibonacci_goal_bus_to_ros(BytesView body) {
  auto bus = parse_pb<example_interfaces::action::v1::FibonacciGoal>(body);
  example_interfaces::action::Fibonacci::Goal ros;
  ros.order = bus.order();
  return ros;
}

example_interfaces::action::Fibonacci::Feedback fibonacci_feedback_bus_to_ros(BytesView body) {
  auto bus = parse_pb<example_interfaces::action::v1::FibonacciFeedback>(body);
  example_interfaces::action::Fibonacci::Feedback ros;
  ros.sequence.assign(bus.sequence().begin(), bus.sequence().end());
  return ros;
}

std::vector<uint8_t> fibonacci_feedback_ros_to_bus(
    const example_interfaces::action::Fibonacci::Feedback &ros) {
  example_interfaces::action::v1::FibonacciFeedback bus;
  for (auto v : ros.sequence) {
    bus.add_sequence(v);
  }
  return encode_pb(bus);
}

example_interfaces::action::Fibonacci::Result fibonacci_result_bus_to_ros(BytesView body) {
  auto bus = parse_pb<example_interfaces::action::v1::FibonacciResult>(body);
  example_interfaces::action::Fibonacci::Result ros;
  ros.sequence.assign(bus.sequence().begin(), bus.sequence().end());
  return ros;
}

std::vector<uint8_t> fibonacci_result_ros_to_bus(
    const example_interfaces::action::Fibonacci::Result &ros) {
  example_interfaces::action::v1::FibonacciResult bus;
  for (auto v : ros.sequence) {
    bus.add_sequence(v);
  }
  return encode_pb(bus);
}

Node make_bus_node(const BuilderState &state) {
  const std::string bus_name = state.name + "_bus";
  switch (state.bus.kind) {
    case BusTransportKind::Tcp:
      return Node::tcp(bus_name, state.bus.host.c_str());
    case BusTransportKind::Ipc:
      return Node::ipc(bus_name, nullptr);
    case BusTransportKind::IpcAt:
      return Node::ipc(bus_name, state.bus.ipc_path.c_str());
    case BusTransportKind::Discover: {
      RobotBusDiscoverOpts opts{};
      opts.api_url = state.bus.api_url.empty() ? nullptr : state.bus.api_url.c_str();
      opts.broker_id = state.bus.broker_id.empty() ? nullptr : state.bus.broker_id.c_str();
      opts.timeout_secs = state.bus.discover_timeout_secs;
      return Node::discover(bus_name, "tcp", &opts);
    }
  }
  throw Error("invalid bus transport");
}

bool should_enable_ros_subscription(bool lazy, std::optional<bool> console_live,
                                    uint32_t subscribers) {
  if (!lazy) {
    return true;
  }
  if (!console_live.has_value()) {
    return false;
  }
  if (!*console_live) {
    return true;
  }
  return subscribers > 0;
}

std::string topic_type_name(const TopicRouteSpec &route) {
  if (route.is_custom()) {
    return "custom";
  }
  switch (route.builtin) {
    case TopicBuiltin::StdMsgsString:
      return "std_msgs/msg/String";
    case TopicBuiltin::SensorMsgsImage:
      return "sensor_msgs/msg/Image";
  }
  return "";
}

std::string service_type_name(const ServiceRouteSpec &route) {
  if (route.is_custom()) {
    return route.custom->type_name();
  }
  switch (route.builtin) {
    case ServiceBuiltin::Trigger:
      return "std_srvs/srv/Trigger";
    case ServiceBuiltin::SetBool:
      return "std_srvs/srv/SetBool";
  }
  return "";
}

std::string action_type_name(const ActionRouteSpec &route) {
  if (route.is_custom()) {
    return route.custom->type_name();
  }
  return "example_interfaces/action/Fibonacci";
}

ConsoleRoute make_topic_console_route(const TopicRouteSpec &route,
                                      std::shared_ptr<RouteHealth> health) {
  return ConsoleRoute{"topic",
                      direction_label(route.direction),
                      route.ros_topic,
                      route.bus_topic,
                      topic_type_name(route),
                      qos_console_label(route.ros_qos),
                      qos_console_label(route.bus_qos),
                      route.lazy,
                      true,
                      std::move(health)};
}

void log_route_table(const std::string &name, const std::vector<ConsoleRoute> &routes) {
  std::string block = "ros2_bridge '" + name + "' routes:";
  for (const auto &r : routes) {
    const auto &source = r.direction == "ros→bus" ? r.ros_name : r.bus_name;
    const auto &target = r.direction == "ros→bus" ? r.bus_name : r.ros_name;
    block += "\n  " + r.kind + "  " + r.direction + "  " + source + " → " + target +
             "  " + (r.type_name.empty() ? "-" : r.type_name) + "  ros=" + r.ros_qos +
             "  bus=" + r.bus_qos + (r.lazy ? "  lazy" : "");
  }
  RCLCPP_INFO(ros2_bridge_logger(), "%s", block.c_str());
}

rclcpp::QoS topic_ros_qos(const TopicQos &qos) {
  rclcpp::QoS out(qos.depth() < 0 ? 0 : qos.depth());
  if (qos.is_best_effort()) {
    out.best_effort();
  } else {
    out.reliable();
  }
  if (qos.is_transient_local()) {
    out.transient_local();
  } else {
    out.durability_volatile();
  }
  return out;
}

TopicPublisher make_bus_publisher(Node &bus_node, const TopicRouteSpec &route) {
  return bus_node.create_publisher(route.bus_topic.c_str(), route.bus_qos.depth());
}

template <typename RosMsg>
rclcpp::SubscriptionBase::SharedPtr make_ros2_to_bus_sub(
    rclcpp::Node::SharedPtr ros_node, const std::string &ros_topic, const rclcpp::QoS &qos,
    std::shared_ptr<TopicPublisher> pub, std::shared_ptr<std::mutex> mtx,
    std::vector<uint8_t> (*convert)(const RosMsg &), std::shared_ptr<DropStats> stats,
    std::shared_ptr<RouteHealth> health) {
  return ros_node->create_subscription<RosMsg>(
      ros_topic, qos,
      [pub, mtx, convert, stats, health, topic = ros_topic](typename RosMsg::ConstSharedPtr msg) {
        forward_ros_to_bus(
            stats, topic, [convert, msg]() { return convert(*msg); },
            [pub, mtx](const std::vector<uint8_t> &bytes) {
              std::lock_guard<std::mutex> lock(*mtx);
              pub->publish(bytes);
            },
            health);
      });
}

template <typename RosMsg>
void wire_topic_ros_to_bus(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                           const TopicRouteSpec &route,
                           std::vector<uint8_t> (*convert)(const RosMsg &),
                           std::vector<rclcpp::SubscriptionBase::SharedPtr> &subs,
                           std::vector<std::shared_ptr<TopicPublisher>> &pubs,
                           std::vector<std::shared_ptr<std::mutex>> &pub_mutexes,
                           std::vector<LazyTopic> &lazy_routes,
                           std::unordered_set<std::string> &eager_bus_topics,
                           std::shared_ptr<DropStats> stats,
                           std::shared_ptr<RouteHealth> health) {
  auto pub = std::make_shared<TopicPublisher>(make_bus_publisher(bus_node, route));
  auto mtx = std::make_shared<std::mutex>();
  auto qos = topic_ros_qos(route.ros_qos);
  auto create = [ros_node, ros_topic = route.ros_topic, qos, pub, mtx, convert, stats, health]() {
    return make_ros2_to_bus_sub<RosMsg>(ros_node, ros_topic, qos, pub, mtx, convert, stats, health);
  };
  if (route.lazy) {
    lazy_routes.push_back(LazyTopic{route.bus_topic, std::move(create), nullptr});
  } else {
    subs.push_back(create());
    eager_bus_topics.insert(route.bus_topic);
  }
  pubs.push_back(std::move(pub));
  pub_mutexes.push_back(std::move(mtx));
}

template <typename RosMsg>
void wire_topic_bus_to_ros(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                           const TopicRouteSpec &route,
                           RosMsg (*convert)(const uint8_t *, size_t),
                           std::vector<rclcpp::PublisherBase::SharedPtr> &pubs,
                           std::vector<std::shared_ptr<void>> &keep_alive,
                           std::shared_ptr<DropStats> stats,
                           std::shared_ptr<RouteHealth> health) {
  auto qos = topic_ros_qos(route.ros_qos);
  auto ros_pub = ros_node->create_publisher<RosMsg>(route.ros_topic, qos);
  auto weak_pub = std::weak_ptr<typename rclcpp::Publisher<RosMsg>>(ros_pub);
  auto topic = route.ros_topic;
  keep_alive.push_back(std::make_shared<SubscriptionHandle>(bus_node.create_subscription(
      route.bus_topic.c_str(),
      [weak_pub, convert, stats, health, topic](BytesView payload) {
        auto pub = weak_pub.lock();
        if (!pub) {
          return;
        }
        forward_bus_to_ros(
            stats, topic,
            [convert, payload]() { return convert(payload.data, payload.size); },
            [pub](RosMsg msg) { pub->publish(std::move(msg)); }, health);
      },
      nullptr, route.bus_qos.depth())));
  pubs.push_back(std::move(ros_pub));
}

void wire_topic_builtin(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                        const TopicRouteSpec &route,
                        std::vector<rclcpp::SubscriptionBase::SharedPtr> &ros_subs,
                        std::vector<rclcpp::PublisherBase::SharedPtr> &ros_pubs,
                        std::vector<std::shared_ptr<TopicPublisher>> &bus_pubs,
                        std::vector<std::shared_ptr<std::mutex>> &bus_pub_mutexes,
                        std::vector<std::shared_ptr<void>> &keep_alive,
                        std::vector<LazyTopic> &lazy_routes,
                        std::unordered_set<std::string> &eager_bus_topics,
                        std::shared_ptr<DropStats> stats,
                        std::shared_ptr<RouteHealth> health) {
  if (route.direction == Direction::Ros2ToBus) {
    switch (route.builtin) {
      case TopicBuiltin::StdMsgsString:
        wire_topic_ros_to_bus<std_msgs::msg::String>(ros_node, bus_node, route, string_ros_to_bus,
                                                     ros_subs, bus_pubs, bus_pub_mutexes,
                                                     lazy_routes, eager_bus_topics, stats, health);
        break;
      case TopicBuiltin::SensorMsgsImage:
        wire_topic_ros_to_bus<sensor_msgs::msg::Image>(ros_node, bus_node, route, image_ros_to_bus,
                                                       ros_subs, bus_pubs, bus_pub_mutexes,
                                                       lazy_routes, eager_bus_topics, stats, health);
        break;
    }
  } else {
    switch (route.builtin) {
      case TopicBuiltin::StdMsgsString:
        wire_topic_bus_to_ros<std_msgs::msg::String>(ros_node, bus_node, route, string_bus_to_ros,
                                                     ros_pubs, keep_alive, stats, health);
        break;
      case TopicBuiltin::SensorMsgsImage:
        wire_topic_bus_to_ros<sensor_msgs::msg::Image>(ros_node, bus_node, route, image_bus_to_ros,
                                                       ros_pubs, keep_alive, stats, health);
        break;
    }
  }
}

void wire_topic(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const TopicRouteSpec &route,
                std::vector<rclcpp::SubscriptionBase::SharedPtr> &ros_subs,
                std::vector<rclcpp::PublisherBase::SharedPtr> &ros_pubs,
                std::vector<std::shared_ptr<TopicPublisher>> &bus_pubs,
                std::vector<std::shared_ptr<std::mutex>> &bus_pub_mutexes,
                std::vector<std::shared_ptr<void>> &keep_alive,
                std::vector<LazyTopic> &lazy_routes,
                std::unordered_set<std::string> &eager_bus_topics,
                std::shared_ptr<DropStats> stats, std::shared_ptr<RouteHealth> health) {
  if (route.is_custom()) {
    keep_alive.push_back(std::shared_ptr<void>(route.custom));
    if (route.lazy) {
      auto pub = std::make_shared<TopicPublisher>(make_bus_publisher(bus_node, route));
      auto mtx = std::make_shared<std::mutex>();
      auto mapper = route.custom;
      auto ros_topic = route.ros_topic;
      auto qos = topic_ros_qos(route.ros_qos);
      bus_pubs.push_back(pub);
      bus_pub_mutexes.push_back(mtx);
      lazy_routes.push_back(LazyTopic{
          route.bus_topic,
          [ros_node, mapper, ros_topic, pub, mtx, qos, stats, health]() {
            return mapper->create_ros2_to_bus_subscription(ros_node, ros_topic, pub, mtx, qos,
                                                           stats, health);
          },
          nullptr});
      return;
    }
    TopicWireContext ctx{ros_node,
                         bus_node,
                         route.ros_topic,
                         route.bus_topic,
                         route.direction,
                         topic_ros_qos(route.ros_qos),
                         route.bus_qos.depth(),
                         keep_alive,
                         stats,
                         health};
    route.custom->attach(ctx);
    if (route.direction == Direction::Ros2ToBus) {
      eager_bus_topics.insert(route.bus_topic);
    }
    return;
  }
  wire_topic_builtin(ros_node, bus_node, route, ros_subs, ros_pubs, bus_pubs, bus_pub_mutexes,
                     keep_alive, lazy_routes, eager_bus_topics, stats, health);
}

struct BuiltinTrigger : TypedServiceMapper<BuiltinTrigger, std_srvs::srv::Trigger> {
  const char *type_name() const override { return "std_srvs/srv/Trigger"; }
  auto ros_req_to_bus(const Request &) const { return trigger_req_to_bus(); }
  Request bus_req_to_ros(BytesView body) const {
    (void)parse_pb<std_srvs::srv::v1::TriggerRequest>(body);
    return Request{};
  }
  auto ros_resp_to_bus(const Response &value) const { return trigger_resp_ros_to_bus(value); }
  auto bus_resp_to_ros(BytesView value) const { return trigger_resp_bus_to_ros(value); }
  Response error_response(const std::string &message) const {
    Response out; out.success = false; out.message = message; return out;
  }
};
struct BuiltinSetBool : TypedServiceMapper<BuiltinSetBool, std_srvs::srv::SetBool> {
  const char *type_name() const override { return "std_srvs/srv/SetBool"; }
  auto ros_req_to_bus(const Request &value) const { return set_bool_req_ros_to_bus(value); }
  auto bus_req_to_ros(BytesView value) const { return set_bool_req_bus_to_ros(value); }
  auto ros_resp_to_bus(const Response &value) const { return set_bool_resp_ros_to_bus(value); }
  auto bus_resp_to_ros(BytesView value) const { return set_bool_resp_bus_to_ros(value); }
  Response error_response(const std::string &message) const {
    Response out; out.success = false; out.message = message; return out;
  }
};
struct BuiltinFibonacci : TypedActionMapper<BuiltinFibonacci, example_interfaces::action::Fibonacci> {
  const char *type_name() const override { return "example_interfaces/action/Fibonacci"; }
  auto ros_goal_to_bus(const Goal &value) const { return fibonacci_goal_ros_to_bus(value); }
  auto bus_goal_to_ros(BytesView value) const { return fibonacci_goal_bus_to_ros(value); }
  auto ros_feedback_to_bus(const Feedback &value) const { return fibonacci_feedback_ros_to_bus(value); }
  auto bus_feedback_to_ros(BytesView value) const { return fibonacci_feedback_bus_to_ros(value); }
  auto ros_result_to_bus(const Result &value) const { return fibonacci_result_ros_to_bus(value); }
  auto bus_result_to_ros(BytesView value) const { return fibonacci_result_bus_to_ros(value); }
};

void wire_service(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ServiceRouteSpec &route,
                  rclcpp::CallbackGroup::SharedPtr group,
                  std::vector<rclcpp::ServiceBase::SharedPtr> &,
                  std::vector<rclcpp::ClientBase::SharedPtr> &,
                  std::vector<std::shared_ptr<ServiceClient>> &,
                  std::vector<std::shared_ptr<void>> &keep_alive) {
  std::shared_ptr<ServiceMapper> mapper = route.custom;
  if (!mapper) {
    if (route.builtin == ServiceBuiltin::Trigger) mapper = std::make_shared<BuiltinTrigger>();
    else mapper = std::make_shared<BuiltinSetBool>();
  }
  keep_alive.push_back(mapper);
  ServiceWireContext ctx{ros_node, bus_node, route.ros_service, route.bus_service,
      route.direction, route.timeout_secs, route.ros_qos, route.bus_qos, group, keep_alive, route.health};
  mapper->attach(ctx);
}

void wire_action(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
                 rclcpp::CallbackGroup::SharedPtr group,
                 std::vector<std::shared_ptr<rclcpp_action::ServerBase>> &,
                 std::vector<std::shared_ptr<rclcpp_action::ClientBase>> &,
                 std::vector<std::shared_ptr<ActionClient>> &,
                 std::vector<std::shared_ptr<void>> &keep_alive) {
  std::shared_ptr<ActionMapper> mapper = route.custom;
  if (!mapper) mapper = std::make_shared<BuiltinFibonacci>();
  keep_alive.push_back(mapper);
  ActionWireContext ctx{ros_node, bus_node, route.ros_action, route.bus_action,
      route.direction, route.timeout_secs, route.ros_qos, route.bus_qos, group, keep_alive, route.health};
  mapper->attach(ctx);
}


}  // namespace ros2_bridge_detail
}  // namespace robot_bus
