#pragma once

#include <robot_bus/ros2_bridge.hpp>

#include <functional>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <unordered_set>
#include <vector>

#include <rclcpp/rclcpp.hpp>
#include <rclcpp_action/rclcpp_action.hpp>

namespace robot_bus {
namespace ros2_bridge_detail {

using detail::ActionRouteSpec;
using detail::BuilderState;
using detail::ServiceRouteSpec;
using detail::TopicRouteSpec;

constexpr double kConsoleDetectTimeoutSecs = 2.0;
constexpr double kIdleGraceSecs = 15.0;
constexpr const char *kTopicDemand = "/robot_bus/topic_demand";
constexpr const char *kTopicsSnapshot = "/robot_bus/topics";
constexpr const char *kBridges = "/robot_bus/bridges";
constexpr const char *kEvents = "/robot_bus/events";

using CreateRosSub = std::function<rclcpp::SubscriptionBase::SharedPtr()>;

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

struct LazyTopic {
  std::string bus_topic;
  CreateRosSub create;
  rclcpp::SubscriptionBase::SharedPtr sub;
};

Node make_bus_node(const BuilderState &state);

bool should_enable_ros_subscription(bool lazy, std::optional<bool> console_live,
                                    uint32_t subscribers);

std::string service_type_name(const ServiceRouteSpec &route);
std::string action_type_name(const ActionRouteSpec &route);

ConsoleRoute make_topic_console_route(const TopicRouteSpec &route,
                                        std::shared_ptr<RouteHealth> health);

void log_route_table(const std::string &name, const std::vector<ConsoleRoute> &routes);

void wire_topic(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const TopicRouteSpec &route,
                std::vector<rclcpp::SubscriptionBase::SharedPtr> &ros_subs,
                std::vector<rclcpp::PublisherBase::SharedPtr> &ros_pubs,
                std::vector<std::shared_ptr<TopicPublisher>> &bus_pubs,
                std::vector<std::shared_ptr<std::mutex>> &bus_pub_mutexes,
                std::vector<std::shared_ptr<void>> &keep_alive,
                std::vector<LazyTopic> &lazy_routes,
                std::unordered_set<std::string> &eager_bus_topics,
                std::shared_ptr<DropStats> stats, std::shared_ptr<RouteHealth> health);

void wire_service(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ServiceRouteSpec &route,
                  rclcpp::CallbackGroup::SharedPtr group,
                  std::vector<rclcpp::ServiceBase::SharedPtr> &ros_srvs,
                  std::vector<rclcpp::ClientBase::SharedPtr> &ros_clients,
                  std::vector<std::shared_ptr<ServiceClient>> &bus_clients,
                  std::vector<std::shared_ptr<void>> &keep_alive);

void wire_action(rclcpp::Node::SharedPtr ros_node, Node &bus_node, const ActionRouteSpec &route,
                 rclcpp::CallbackGroup::SharedPtr group,
                 std::vector<std::shared_ptr<rclcpp_action::ServerBase>> &ros_actions,
                 std::vector<std::shared_ptr<rclcpp_action::ClientBase>> &ros_action_clients,
                 std::vector<std::shared_ptr<ActionClient>> &bus_action_clients,
                 std::vector<std::shared_ptr<void>> &keep_alive);

}  // namespace ros2_bridge_detail
}  // namespace robot_bus
