#include <robot_bus/ros2_bridge.hpp>
#include <robot_bus/typed.hpp>

#include <robot_bus/builtin_interfaces/msg/v1/time.pb.h>
#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>
#include <robot_bus/sensor_msgs/msg/v1/image.pb.h>
#include <robot_bus/robot_bus_interfaces/msg/v1/console_status.pb.h>
#include <robot_bus/std_msgs/msg/v1/header.pb.h>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>
#include <robot_bus/std_srvs/srv/v1/trigger.pb.h>

#include <example_interfaces/action/fibonacci.hpp>
#include <rclcpp/rclcpp.hpp>
#include <rclcpp_action/rclcpp_action.hpp>
#include <sensor_msgs/msg/image.hpp>
#include <std_msgs/msg/string.hpp>
#include <std_srvs/srv/set_bool.hpp>
#include <std_srvs/srv/trigger.hpp>

#include <atomic>
#include <chrono>
#include <functional>
#include <future>
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
namespace {

using detail::ActionBuiltin;
using detail::ActionRouteSpec;
using detail::BusTransportKind;
using detail::BuilderState;
using detail::ServiceBuiltin;
using detail::ServiceRouteSpec;
using detail::TopicBuiltin;
using detail::TopicRouteSpec;

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

std::vector<uint8_t> fibonacci_empty_result() {
  return encode_pb(example_interfaces::action::v1::FibonacciResult{});
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

constexpr double kConsoleDetectTimeoutSecs = 2.0;
constexpr double kIdleGraceSecs = 15.0;
constexpr const char *kTopicDemand = "/robot_bus/topic_demand";
constexpr const char *kTopicsSnapshot = "/robot_bus/topics";
constexpr const char *kBridges = "/robot_bus/bridges";
constexpr const char *kEvents = "/robot_bus/events";

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

struct ConsoleRoute {
  std::string kind;
  std::string direction;
  std::string ros_name;
  std::string bus_name;
  std::string type_name;
  std::string ros_qos;
  std::string bus_qos;
  bool lazy = false;
  bool watch_idle = false;
  std::shared_ptr<RouteHealth> health;
};

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
    block += "\n  " + r.kind + "  " + r.direction + "  " + r.ros_name + " → " + r.bus_name +
             "  " + (r.type_name.empty() ? "-" : r.type_name) + "  ros=" + r.ros_qos +
             "  bus=" + r.bus_qos + (r.lazy ? "  lazy" : "");
  }
  RCLCPP_INFO(ros2_bridge_logger(), "%s", block.c_str());
}

using CreateRosSub = std::function<rclcpp::SubscriptionBase::SharedPtr()>;

struct LazyTopic {
  std::string bus_topic;
  CreateRosSub create;
  rclcpp::SubscriptionBase::SharedPtr sub;
};

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

void wire_trigger(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ServiceRouteSpec &route,
                  rclcpp::CallbackGroup::SharedPtr group,
                  std::vector<rclcpp::ServiceBase::SharedPtr> &ros_srvs,
                  std::vector<rclcpp::ClientBase::SharedPtr> &ros_clients,
                  std::vector<std::shared_ptr<ServiceClient>> &bus_clients,
                  std::vector<std::shared_ptr<void>> &keep_alive) {
  const double timeout = route.timeout_secs;
  if (route.direction == Direction::Ros2ToBus) {
    auto bus_client =
        std::make_shared<ServiceClient>(bus_node.create_client(route.bus_service.c_str(),
                                                              route.bus_qos.depth()));
    auto mtx = std::make_shared<std::mutex>();
    auto srv = ros_node->create_service<std_srvs::srv::Trigger>(
        route.ros_service,
        [bus_client, mtx, timeout](const std::shared_ptr<std_srvs::srv::Trigger::Request>,
                                   std::shared_ptr<std_srvs::srv::Trigger::Response> response) {
          try {
            auto req_bytes = trigger_req_to_bus();
            std::vector<uint8_t> resp_bytes;
            {
              std::lock_guard<std::mutex> lock(*mtx);
              resp_bytes = bus_client->call(req_bytes, timeout);
            }
            *response = trigger_resp_bus_to_ros(resp_bytes);
          } catch (const std::exception &e) {
            response->success = false;
            response->message = std::string("bus call failed: ") + e.what();
          } catch (...) {
            response->success = false;
            response->message = "bus call failed";
          }
        },
        service_rmw_qos(route.ros_qos), group);
    bus_clients.push_back(std::move(bus_client));
    ros_srvs.push_back(std::move(srv));
  } else {
    auto ros_client = ros_node->create_client<std_srvs::srv::Trigger>(
        route.ros_service, service_rmw_qos(route.ros_qos), group);
    ros_clients.push_back(ros_client);
    keep_alive.push_back(std::make_shared<ServiceHandle>(bus_node.create_service(
        route.bus_service.c_str(),
        [ros_client, timeout](BytesView body) -> std::vector<uint8_t> {
          (void)body;
          if (!ros_client->wait_for_service(std::chrono::duration<double>(timeout))) {
            std_srvs::srv::Trigger::Response err;
            err.success = false;
            err.message = "timed out waiting for ROS service";
            return trigger_resp_ros_to_bus(err);
          }
          auto req = std::make_shared<std_srvs::srv::Trigger::Request>();
          auto future = ros_client->async_send_request(req);
          const auto status = future.wait_for(std::chrono::duration<double>(timeout));
          if (status != std::future_status::ready) {
            std_srvs::srv::Trigger::Response err;
            err.success = false;
            err.message = "timed out waiting for ROS response";
            return trigger_resp_ros_to_bus(err);
          }
          return trigger_resp_ros_to_bus(*future.get());
        },
        nullptr, route.bus_qos.depth())));
  }
}

void wire_set_bool(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ServiceRouteSpec &route,
                   rclcpp::CallbackGroup::SharedPtr group,
                   std::vector<rclcpp::ServiceBase::SharedPtr> &ros_srvs,
                   std::vector<rclcpp::ClientBase::SharedPtr> &ros_clients,
                   std::vector<std::shared_ptr<ServiceClient>> &bus_clients,
                   std::vector<std::shared_ptr<void>> &keep_alive) {
  const double timeout = route.timeout_secs;
  if (route.direction == Direction::Ros2ToBus) {
    auto bus_client =
        std::make_shared<ServiceClient>(bus_node.create_client(route.bus_service.c_str(),
                                                              route.bus_qos.depth()));
    auto mtx = std::make_shared<std::mutex>();
    auto srv = ros_node->create_service<std_srvs::srv::SetBool>(
        route.ros_service,
        [bus_client, mtx, timeout](const std::shared_ptr<std_srvs::srv::SetBool::Request> request,
                                   std::shared_ptr<std_srvs::srv::SetBool::Response> response) {
          try {
            auto req_bytes = set_bool_req_ros_to_bus(*request);
            std::vector<uint8_t> resp_bytes;
            {
              std::lock_guard<std::mutex> lock(*mtx);
              resp_bytes = bus_client->call(req_bytes, timeout);
            }
            *response = set_bool_resp_bus_to_ros(resp_bytes);
          } catch (const std::exception &e) {
            response->success = false;
            response->message = std::string("bus call failed: ") + e.what();
          } catch (...) {
            response->success = false;
            response->message = "bus call failed";
          }
        },
        service_rmw_qos(route.ros_qos), group);
    bus_clients.push_back(std::move(bus_client));
    ros_srvs.push_back(std::move(srv));
  } else {
    auto ros_client = ros_node->create_client<std_srvs::srv::SetBool>(
        route.ros_service, service_rmw_qos(route.ros_qos), group);
    ros_clients.push_back(ros_client);
    keep_alive.push_back(std::make_shared<ServiceHandle>(bus_node.create_service(
        route.bus_service.c_str(),
        [ros_client, timeout](BytesView body) -> std::vector<uint8_t> {
          if (!ros_client->wait_for_service(std::chrono::duration<double>(timeout))) {
            std_srvs::srv::SetBool::Response err;
            err.success = false;
            err.message = "timed out waiting for ROS service";
            return set_bool_resp_ros_to_bus(err);
          }
          auto req = std::make_shared<std_srvs::srv::SetBool::Request>(set_bool_req_bus_to_ros(body));
          auto future = ros_client->async_send_request(req);
          const auto status = future.wait_for(std::chrono::duration<double>(timeout));
          if (status != std::future_status::ready) {
            std_srvs::srv::SetBool::Response err;
            err.success = false;
            err.message = "timed out waiting for ROS response";
            return set_bool_resp_ros_to_bus(err);
          }
          return set_bool_resp_ros_to_bus(*future.get());
        },
        nullptr, route.bus_qos.depth())));
  }
}

void wire_service_builtin(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                          const ServiceRouteSpec &route, rclcpp::CallbackGroup::SharedPtr group,
                          std::vector<rclcpp::ServiceBase::SharedPtr> &ros_srvs,
                          std::vector<rclcpp::ClientBase::SharedPtr> &ros_clients,
                          std::vector<std::shared_ptr<ServiceClient>> &bus_clients,
                          std::vector<std::shared_ptr<void>> &keep_alive) {
  switch (route.builtin) {
    case ServiceBuiltin::Trigger:
      wire_trigger(ros_node, bus_node, route, group, ros_srvs, ros_clients, bus_clients,
                   keep_alive);
      break;
    case ServiceBuiltin::SetBool:
      wire_set_bool(ros_node, bus_node, route, group, ros_srvs, ros_clients, bus_clients,
                    keep_alive);
      break;
  }
}

void wire_service(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ServiceRouteSpec &route,
                  rclcpp::CallbackGroup::SharedPtr group,
                  std::vector<rclcpp::ServiceBase::SharedPtr> &ros_srvs,
                  std::vector<rclcpp::ClientBase::SharedPtr> &ros_clients,
                  std::vector<std::shared_ptr<ServiceClient>> &bus_clients,
                  std::vector<std::shared_ptr<void>> &keep_alive) {
  if (route.is_custom()) {
    keep_alive.push_back(std::shared_ptr<void>(route.custom));
    ServiceWireContext ctx{ros_node,        bus_node,           route.ros_service, route.bus_service,
                           route.direction, route.timeout_secs, route.ros_qos,     route.bus_qos,
                           group,           keep_alive};
    route.custom->attach(ctx);
    return;
  }
  wire_service_builtin(ros_node, bus_node, route, group, ros_srvs, ros_clients, bus_clients,
                       keep_alive);
}

using Fibonacci = example_interfaces::action::Fibonacci;
using GoalHandleFibonacci = rclcpp_action::ServerGoalHandle<Fibonacci>;

void wire_fibonacci_ros_to_bus(
    rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
    rclcpp::CallbackGroup::SharedPtr group,
    std::vector<std::shared_ptr<rclcpp_action::ServerBase>> &ros_actions,
    std::vector<std::shared_ptr<ActionClient>> &bus_action_clients) {
  auto bus_client =
      std::make_shared<ActionClient>(
          bus_node.create_action_client(route.bus_action.c_str(), route.bus_qos.depth()));
  auto mtx = std::make_shared<std::mutex>();
  auto live = std::make_shared<std::mutex>();
  auto bus_goals = std::make_shared<
      std::unordered_map<const void *, std::shared_ptr<ActionGoalHandle>>>();
  const double timeout = route.timeout_secs;

  auto handle_goal = [](const rclcpp_action::GoalUUID &,
                        std::shared_ptr<const Fibonacci::Goal>) {
    return rclcpp_action::GoalResponse::ACCEPT_AND_EXECUTE;
  };
  auto handle_cancel = [live, bus_goals](const std::shared_ptr<GoalHandleFibonacci> gh) {
    std::lock_guard<std::mutex> lock(*live);
    auto it = bus_goals->find(gh.get());
    if (it != bus_goals->end() && it->second) {
      try {
        it->second->cancel();
      } catch (...) {
      }
    }
    return rclcpp_action::CancelResponse::ACCEPT;
  };
  auto handle_accepted = [bus_client, mtx, timeout, live, bus_goals](
                             const std::shared_ptr<GoalHandleFibonacci> goal_handle) {
    std::thread([bus_client, mtx, timeout, live, bus_goals, goal_handle]() {
      const auto goal = goal_handle->get_goal();
      try {
        auto goal_bytes = fibonacci_goal_ros_to_bus(*goal);
        auto handle = std::make_shared<ActionGoalHandle>([&]() {
          std::lock_guard<std::mutex> lock(*mtx);
          return bus_client->send_goal(
              goal_bytes,
              [goal_handle](const ActionMessage &message) {
                if (message.kind != "FEEDBACK") {
                  return;
                }
                try {
                  auto feedback = std::make_shared<Fibonacci::Feedback>(
                      fibonacci_feedback_bus_to_ros(message.body));
                  goal_handle->publish_feedback(feedback);
                } catch (...) {
                }
              },
              nullptr, timeout);
        }());
        {
          std::lock_guard<std::mutex> lock(*live);
          (*bus_goals)[goal_handle.get()] = handle;
        }
        auto result_msg = handle->wait_result(timeout);
        if (result_msg.kind != "RESULT") {
          if (goal_handle->is_canceling()) {
            goal_handle->canceled(std::make_shared<Fibonacci::Result>());
          } else {
            goal_handle->abort(std::make_shared<Fibonacci::Result>());
          }
        } else {
          auto result =
              std::make_shared<Fibonacci::Result>(fibonacci_result_bus_to_ros(result_msg.body));
          if (goal_handle->is_canceling()) {
            goal_handle->canceled(result);
          } else {
            goal_handle->succeed(result);
          }
        }
      } catch (...) {
        if (goal_handle->is_canceling()) {
          goal_handle->canceled(std::make_shared<Fibonacci::Result>());
        } else {
          goal_handle->abort(std::make_shared<Fibonacci::Result>());
        }
      }
      std::lock_guard<std::mutex> lock(*live);
      bus_goals->erase(goal_handle.get());
    }).detach();
  };

  auto server = rclcpp_action::create_server<Fibonacci>(
      ros_node, route.ros_action, handle_goal, handle_cancel, handle_accepted,
      action_server_qos(route.ros_qos), group);
  bus_action_clients.push_back(std::move(bus_client));
  ros_actions.push_back(std::move(server));
}

void wire_fibonacci_bus_to_ros(
    rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
    rclcpp::CallbackGroup::SharedPtr group,
    std::vector<std::shared_ptr<rclcpp_action::ClientBase>> &ros_action_clients,
    std::vector<std::shared_ptr<void>> &keep_alive) {
  auto ros_client = rclcpp_action::create_client<Fibonacci>(
      ros_node, route.ros_action, group, action_client_qos(route.ros_qos));
  auto mtx = std::make_shared<std::mutex>();
  const double timeout = route.timeout_secs;
  ros_action_clients.push_back(ros_client);

  keep_alive.push_back(std::make_shared<ActionServerHandle>(bus_node.create_action_server_live(
      route.bus_action.c_str(),
      [ros_client, mtx, timeout](BytesView body, const ActionGoalContext &actx)
          -> std::vector<uint8_t> {
        Fibonacci::Goal goal;
        try {
          goal = fibonacci_goal_bus_to_ros(body);
        } catch (...) {
          return fibonacci_empty_result();
        }

        if (!ros_client->wait_for_action_server(std::chrono::duration<double>(timeout))) {
          return fibonacci_empty_result();
        }

        typename rclcpp_action::Client<Fibonacci>::SendGoalOptions opts;
        opts.feedback_callback =
            [actx](rclcpp_action::ClientGoalHandle<Fibonacci>::SharedPtr,
                   const std::shared_ptr<const Fibonacci::Feedback> feedback) {
              try {
                actx.publish_feedback(fibonacci_feedback_ros_to_bus(*feedback));
              } catch (...) {
              }
            };

        std::shared_future<rclcpp_action::ClientGoalHandle<Fibonacci>::SharedPtr> goal_future;
        {
          std::lock_guard<std::mutex> lock(*mtx);
          goal_future = ros_client->async_send_goal(goal, opts);
        }
        if (goal_future.wait_for(std::chrono::duration<double>(timeout)) !=
            std::future_status::ready) {
          return fibonacci_empty_result();
        }
        auto goal_handle = goal_future.get();
        if (!goal_handle) {
          return fibonacci_empty_result();
        }

        auto result_future = ros_client->async_get_result(goal_handle);
        const auto deadline =
            std::chrono::steady_clock::now() + std::chrono::duration<double>(timeout);
        bool cancel_sent = false;
        while (result_future.wait_for(std::chrono::milliseconds(20)) !=
               std::future_status::ready) {
          if (actx.cancel_requested() && !cancel_sent) {
            ros_client->async_cancel_goal(goal_handle);
            cancel_sent = true;
          }
          if (std::chrono::steady_clock::now() >= deadline) {
            return fibonacci_empty_result();
          }
        }

        try {
          auto wrapped = result_future.get();
          if (wrapped.result) {
            return fibonacci_result_ros_to_bus(*wrapped.result);
          }
          return fibonacci_empty_result();
        } catch (...) {
          return fibonacci_empty_result();
        }
      },
      nullptr, route.bus_qos.depth())));
}

void wire_action_builtin(
    rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
    rclcpp::CallbackGroup::SharedPtr group,
    std::vector<std::shared_ptr<rclcpp_action::ServerBase>> &ros_actions,
    std::vector<std::shared_ptr<rclcpp_action::ClientBase>> &ros_action_clients,
    std::vector<std::shared_ptr<ActionClient>> &bus_action_clients,
    std::vector<std::shared_ptr<void>> &keep_alive) {
  switch (route.builtin) {
    case ActionBuiltin::Fibonacci:
      if (route.direction == Direction::Ros2ToBus) {
        wire_fibonacci_ros_to_bus(ros_node, bus_node, route, group, ros_actions,
                                  bus_action_clients);
      } else {
        wire_fibonacci_bus_to_ros(ros_node, bus_node, route, group, ros_action_clients,
                                  keep_alive);
      }
      break;
  }
}

void wire_action(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
                 rclcpp::CallbackGroup::SharedPtr group,
                 std::vector<std::shared_ptr<rclcpp_action::ServerBase>> &ros_actions,
                 std::vector<std::shared_ptr<rclcpp_action::ClientBase>> &ros_action_clients,
                 std::vector<std::shared_ptr<ActionClient>> &bus_action_clients,
                 std::vector<std::shared_ptr<void>> &keep_alive) {
  if (route.is_custom()) {
    keep_alive.push_back(std::shared_ptr<void>(route.custom));
    ActionWireContext ctx{ros_node,        bus_node,           route.ros_action,  route.bus_action,
                          route.direction, route.timeout_secs, route.ros_qos,     route.bus_qos,
                          group,           keep_alive};
    route.custom->attach(ctx);
    return;
  }
  wire_action_builtin(ros_node, bus_node, route, group, ros_actions, ros_action_clients,
                      bus_action_clients, keep_alive);
}

}  // namespace

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
          " for 15s; possible wrong direction or ROS QoS mismatch";
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
    wire_topic(impl->ros_node, impl->bus_node, route, impl->ros_subs, impl->ros_pubs,
               impl->bus_pubs, impl->bus_pub_mutexes, impl->keep_alive, impl->lazy_routes,
               impl->eager_bus_topics, impl->drop_stats, health);
    impl->console_routes.push_back(make_topic_console_route(route, health));
  }
  for (const auto &svc : state->services) {
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
        std::make_shared<RouteHealth>()});
  }
  for (const auto &act : state->actions) {
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
        std::make_shared<RouteHealth>()});
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
