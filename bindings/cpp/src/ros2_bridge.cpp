#include <robot_bus/ros2_bridge.hpp>

#include <robot_bus/builtin_interfaces/msg/v1/time.pb.h>
#include <robot_bus/robot_bus_interface/action/v1/fibonacci.pb.h>
#include <robot_bus/sensor_msgs/msg/v1/image.pb.h>
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
#include <future>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
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

std::vector<uint8_t> serialize_pb(const google::protobuf::MessageLite &msg) {
  std::string bytes;
  if (!msg.SerializeToString(&bytes)) {
    throw Error("protobuf SerializeToString failed");
  }
  return std::vector<uint8_t>(bytes.begin(), bytes.end());
}

template <typename T>
T parse_pb(const uint8_t *data, size_t len) {
  T msg;
  if (!msg.ParseFromArray(data, static_cast<int>(len))) {
    throw Error("protobuf ParseFromArray failed");
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
  return serialize_pb(bus);
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
  return serialize_pb(bus);
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
  return serialize_pb(std_srvs::srv::v1::TriggerRequest{});
}

std::vector<uint8_t> trigger_resp_ros_to_bus(const std_srvs::srv::Trigger::Response &ros) {
  std_srvs::srv::v1::TriggerResponse bus;
  bus.set_success(ros.success);
  bus.set_message(ros.message);
  return serialize_pb(bus);
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
  return serialize_pb(bus);
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
  return serialize_pb(bus);
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
  robot_bus_interface::action::v1::FibonacciGoal bus;
  bus.set_order(ros.order);
  return serialize_pb(bus);
}

example_interfaces::action::Fibonacci::Goal fibonacci_goal_bus_to_ros(BytesView body) {
  auto bus = parse_pb<robot_bus_interface::action::v1::FibonacciGoal>(body);
  example_interfaces::action::Fibonacci::Goal ros;
  ros.order = bus.order();
  return ros;
}

example_interfaces::action::Fibonacci::Feedback fibonacci_feedback_bus_to_ros(BytesView body) {
  auto bus = parse_pb<robot_bus_interface::action::v1::FibonacciFeedback>(body);
  example_interfaces::action::Fibonacci::Feedback ros;
  ros.sequence.assign(bus.sequence().begin(), bus.sequence().end());
  return ros;
}

std::vector<uint8_t> fibonacci_feedback_ros_to_bus(
    const example_interfaces::action::Fibonacci::Feedback &ros) {
  robot_bus_interface::action::v1::FibonacciFeedback bus;
  for (auto v : ros.sequence) {
    bus.add_sequence(v);
  }
  return serialize_pb(bus);
}

example_interfaces::action::Fibonacci::Result fibonacci_result_bus_to_ros(BytesView body) {
  auto bus = parse_pb<robot_bus_interface::action::v1::FibonacciResult>(body);
  example_interfaces::action::Fibonacci::Result ros;
  ros.sequence.assign(bus.sequence().begin(), bus.sequence().end());
  return ros;
}

std::vector<uint8_t> fibonacci_result_ros_to_bus(
    const example_interfaces::action::Fibonacci::Result &ros) {
  robot_bus_interface::action::v1::FibonacciResult bus;
  for (auto v : ros.sequence) {
    bus.add_sequence(v);
  }
  return serialize_pb(bus);
}

std::vector<std::pair<std::string, std::vector<uint8_t>>> fibonacci_empty_result_phases() {
  return {{"RESULT", serialize_pb(robot_bus_interface::action::v1::FibonacciResult{})}};
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

template <typename RosMsg>
void wire_topic_ros_to_bus(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                           const TopicRouteSpec &route,
                           std::vector<uint8_t> (*convert)(const RosMsg &),
                           std::vector<rclcpp::SubscriptionBase::SharedPtr> &subs,
                           std::vector<std::shared_ptr<TopicPublisher>> &pubs,
                           std::vector<std::shared_ptr<std::mutex>> &pub_mutexes) {
  auto pub = std::make_shared<TopicPublisher>(bus_node.create_publisher(route.bus_topic.c_str()));
  auto mtx = std::make_shared<std::mutex>();
  auto sub = ros_node->create_subscription<RosMsg>(
      route.ros_topic, rclcpp::QoS(10),
      [pub, mtx, convert](typename RosMsg::ConstSharedPtr msg) {
        try {
          auto bytes = convert(*msg);
          std::lock_guard<std::mutex> lock(*mtx);
          pub->publish(bytes);
        } catch (...) {
          // Drop conversion / publish errors; keep bridge alive.
        }
      });
  pubs.push_back(std::move(pub));
  pub_mutexes.push_back(std::move(mtx));
  subs.push_back(std::move(sub));
}

template <typename RosMsg>
void wire_topic_bus_to_ros(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                           const TopicRouteSpec &route,
                           RosMsg (*convert)(const uint8_t *, size_t),
                           std::vector<rclcpp::PublisherBase::SharedPtr> &pubs,
                           std::vector<std::shared_ptr<void>> &keep_alive) {
  auto ros_pub = ros_node->create_publisher<RosMsg>(route.ros_topic, 10);
  auto weak_pub = std::weak_ptr<typename rclcpp::Publisher<RosMsg>>(ros_pub);
  keep_alive.push_back(std::make_shared<SubscriptionHandle>(bus_node.create_subscription(
      route.bus_topic.c_str(),
      [weak_pub, convert](std::string_view, BytesView payload) {
        auto pub = weak_pub.lock();
        if (!pub) {
          return;
        }
        try {
          pub->publish(convert(payload.data, payload.size));
        } catch (...) {
        }
      })));
  pubs.push_back(std::move(ros_pub));
}

void wire_topic_builtin(rclcpp::Node::SharedPtr ros_node, Node &bus_node,
                        const TopicRouteSpec &route,
                        std::vector<rclcpp::SubscriptionBase::SharedPtr> &ros_subs,
                        std::vector<rclcpp::PublisherBase::SharedPtr> &ros_pubs,
                        std::vector<std::shared_ptr<TopicPublisher>> &bus_pubs,
                        std::vector<std::shared_ptr<std::mutex>> &bus_pub_mutexes,
                        std::vector<std::shared_ptr<void>> &keep_alive) {
  if (route.direction == Direction::Ros2ToBus) {
    switch (route.builtin) {
      case TopicBuiltin::StdMsgsString:
        wire_topic_ros_to_bus<std_msgs::msg::String>(ros_node, bus_node, route, string_ros_to_bus,
                                                     ros_subs, bus_pubs, bus_pub_mutexes);
        break;
      case TopicBuiltin::SensorMsgsImage:
        wire_topic_ros_to_bus<sensor_msgs::msg::Image>(ros_node, bus_node, route, image_ros_to_bus,
                                                       ros_subs, bus_pubs, bus_pub_mutexes);
        break;
    }
  } else {
    switch (route.builtin) {
      case TopicBuiltin::StdMsgsString:
        wire_topic_bus_to_ros<std_msgs::msg::String>(ros_node, bus_node, route, string_bus_to_ros,
                                                     ros_pubs, keep_alive);
        break;
      case TopicBuiltin::SensorMsgsImage:
        wire_topic_bus_to_ros<sensor_msgs::msg::Image>(ros_node, bus_node, route, image_bus_to_ros,
                                                       ros_pubs, keep_alive);
        break;
    }
  }
}

void wire_topic(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const TopicRouteSpec &route,
                std::vector<rclcpp::SubscriptionBase::SharedPtr> &ros_subs,
                std::vector<rclcpp::PublisherBase::SharedPtr> &ros_pubs,
                std::vector<std::shared_ptr<TopicPublisher>> &bus_pubs,
                std::vector<std::shared_ptr<std::mutex>> &bus_pub_mutexes,
                std::vector<std::shared_ptr<void>> &keep_alive) {
  if (route.is_custom()) {
    keep_alive.push_back(std::shared_ptr<void>(route.custom));
    TopicWireContext ctx{ros_node,           bus_node, route.ros_topic, route.bus_topic,
                         route.direction,    keep_alive};
    route.custom->attach(ctx);
    return;
  }
  wire_topic_builtin(ros_node, bus_node, route, ros_subs, ros_pubs, bus_pubs, bus_pub_mutexes,
                     keep_alive);
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
        std::make_shared<ServiceClient>(bus_node.create_client(route.bus_service.c_str()));
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
        rmw_qos_profile_services_default, group);
    bus_clients.push_back(std::move(bus_client));
    ros_srvs.push_back(std::move(srv));
  } else {
    auto ros_client = ros_node->create_client<std_srvs::srv::Trigger>(
        route.ros_service, rmw_qos_profile_services_default, group);
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
        })));
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
        std::make_shared<ServiceClient>(bus_node.create_client(route.bus_service.c_str()));
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
        rmw_qos_profile_services_default, group);
    bus_clients.push_back(std::move(bus_client));
    ros_srvs.push_back(std::move(srv));
  } else {
    auto ros_client = ros_node->create_client<std_srvs::srv::SetBool>(
        route.ros_service, rmw_qos_profile_services_default, group);
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
        })));
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
    ServiceWireContext ctx{ros_node,       bus_node,      route.ros_service, route.bus_service,
                           route.direction, route.timeout_secs, group,         keep_alive};
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
      std::make_shared<ActionClient>(bus_node.create_action_client(route.bus_action.c_str()));
  auto mtx = std::make_shared<std::mutex>();
  const double timeout = route.timeout_secs;

  auto handle_goal = [](const rclcpp_action::GoalUUID &,
                        std::shared_ptr<const Fibonacci::Goal>) {
    return rclcpp_action::GoalResponse::ACCEPT_AND_EXECUTE;
  };
  auto handle_cancel = [](const std::shared_ptr<GoalHandleFibonacci>) {
    return rclcpp_action::CancelResponse::ACCEPT;
  };
  auto handle_accepted = [bus_client, mtx, timeout](
                             const std::shared_ptr<GoalHandleFibonacci> goal_handle) {
    std::thread([bus_client, mtx, timeout, goal_handle]() {
      const auto goal = goal_handle->get_goal();
      try {
        auto goal_bytes = fibonacci_goal_ros_to_bus(*goal);
        ActionGoalHandle handle = [&]() {
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
        }();
        auto result_msg = handle.wait_result(timeout);
        if (result_msg.kind != "RESULT") {
          goal_handle->abort(std::make_shared<Fibonacci::Result>());
          return;
        }
        auto result =
            std::make_shared<Fibonacci::Result>(fibonacci_result_bus_to_ros(result_msg.body));
        goal_handle->succeed(result);
      } catch (...) {
        goal_handle->abort(std::make_shared<Fibonacci::Result>());
      }
    }).detach();
  };

  auto server = rclcpp_action::create_server<Fibonacci>(
      ros_node, route.ros_action, handle_goal, handle_cancel, handle_accepted,
      rcl_action_server_get_default_options(), group);
  bus_action_clients.push_back(std::move(bus_client));
  ros_actions.push_back(std::move(server));
}

void wire_fibonacci_bus_to_ros(
    rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
    rclcpp::CallbackGroup::SharedPtr group,
    std::vector<std::shared_ptr<rclcpp_action::ClientBase>> &ros_action_clients,
    std::vector<std::shared_ptr<void>> &keep_alive) {
  auto ros_client = rclcpp_action::create_client<Fibonacci>(ros_node, route.ros_action, group);
  auto mtx = std::make_shared<std::mutex>();
  const double timeout = route.timeout_secs;
  ros_action_clients.push_back(ros_client);

  keep_alive.push_back(std::make_shared<ActionServerHandle>(bus_node.create_action_server(
      route.bus_action.c_str(),
      [ros_client, mtx, timeout](BytesView body)
          -> std::vector<std::pair<std::string, std::vector<uint8_t>>> {
        Fibonacci::Goal goal;
        try {
          goal = fibonacci_goal_bus_to_ros(body);
        } catch (...) {
          return fibonacci_empty_result_phases();
        }

        if (!ros_client->wait_for_action_server(std::chrono::duration<double>(timeout))) {
          return fibonacci_empty_result_phases();
        }

        auto feedbacks = std::make_shared<std::vector<std::vector<uint8_t>>>();
        auto feedback_mtx = std::make_shared<std::mutex>();

        typename rclcpp_action::Client<Fibonacci>::SendGoalOptions opts;
        opts.feedback_callback =
            [feedbacks, feedback_mtx](
                rclcpp_action::ClientGoalHandle<Fibonacci>::SharedPtr,
                const std::shared_ptr<const Fibonacci::Feedback> feedback) {
              try {
                auto bytes = fibonacci_feedback_ros_to_bus(*feedback);
                std::lock_guard<std::mutex> lock(*feedback_mtx);
                feedbacks->push_back(std::move(bytes));
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
          return fibonacci_empty_result_phases();
        }
        auto goal_handle = goal_future.get();
        if (!goal_handle) {
          return fibonacci_empty_result_phases();
        }

        auto result_future = ros_client->async_get_result(goal_handle);
        if (result_future.wait_for(std::chrono::duration<double>(timeout)) !=
            std::future_status::ready) {
          return fibonacci_empty_result_phases();
        }

        std::vector<std::pair<std::string, std::vector<uint8_t>>> phases;
        {
          std::lock_guard<std::mutex> lock(*feedback_mtx);
          for (auto &fb : *feedbacks) {
            phases.emplace_back("FEEDBACK", std::move(fb));
          }
        }
        try {
          auto wrapped = result_future.get();
          if (wrapped.result) {
            phases.emplace_back("RESULT", fibonacci_result_ros_to_bus(*wrapped.result));
          } else {
            phases.emplace_back("RESULT",
                                serialize_pb(robot_bus_interface::action::v1::FibonacciResult{}));
          }
        } catch (...) {
          return fibonacci_empty_result_phases();
        }
        return phases;
      })));
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
    ActionWireContext ctx{ros_node,        bus_node,       route.ros_action, route.bus_action,
                          route.direction, route.timeout_secs, group,        keep_alive};
    route.custom->attach(ctx);
    return;
  }
  wire_action_builtin(ros_node, bus_node, route, group, ros_actions, ros_action_clients,
                      bus_action_clients, keep_alive);
}

}  // namespace

void TopicMapper::attach(TopicWireContext &ctx) {
  (void)ctx;
  throw Error(std::string("custom TopicMapper must override attach(); type=") + type_name());
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
  std::atomic<bool> halt{false};
  std::thread spin_thread;

  explicit Impl(Node bus) : bus_node(std::move(bus)) {}

  ~Impl() {
    halt.store(true);
    if (spin_thread.joinable()) {
      spin_thread.join();
    }
    if (executor && ros_node) {
      executor->remove_node(ros_node);
    }
  }
};

Ros2Bridge::Ros2Bridge(std::unique_ptr<Impl> impl) : impl_(std::move(impl)) {}

Ros2Bridge::~Ros2Bridge() = default;

Ros2Bridge::Ros2Bridge(Ros2Bridge &&) noexcept = default;

Ros2Bridge &Ros2Bridge::operator=(Ros2Bridge &&) noexcept = default;

void Ros2Bridge::spin() {
  while (true) {
    spin_once(0.01);
  }
}

void Ros2Bridge::spin_once(double timeout_secs) {
  if (!impl_) {
    throw Error("Ros2Bridge is empty");
  }
  impl_->bus_node.spin_once(timeout_secs);
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
    wire_topic(impl->ros_node, impl->bus_node, route, impl->ros_subs, impl->ros_pubs,
               impl->bus_pubs, impl->bus_pub_mutexes, impl->keep_alive);
  }
  for (const auto &svc : state->services) {
    wire_service(impl->ros_node, impl->bus_node, svc, impl->callback_group, impl->ros_srvs,
                 impl->ros_clients, impl->bus_clients, impl->keep_alive);
  }
  for (const auto &act : state->actions) {
    wire_action(impl->ros_node, impl->bus_node, act, impl->callback_group, impl->ros_actions,
                impl->ros_action_clients, impl->bus_action_clients, impl->keep_alive);
  }

  auto *raw = impl.get();
  raw->spin_thread = std::thread([raw]() {
    while (!raw->halt.load()) {
      raw->executor->spin_some(std::chrono::milliseconds(10));
      std::this_thread::sleep_for(std::chrono::milliseconds(1));
    }
  });

  return Ros2Bridge(std::move(impl));
}

}  // namespace robot_bus
