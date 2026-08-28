#pragma once

/// CRTP helpers: custom mappers only implement convert methods; library wires ROS↔bus.
/// Include after `ros2_bridge.hpp` definitions of WireContext / Mapper bases.
/// Requires `ROBOT_BUS_HAS_ROS2`.

#include <chrono>
#include <future>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
#include <utility>
#include <vector>

namespace robot_bus {

/// Topic: Derived provides `ros_to_bus` / `bus_to_ros` (and inherits `type_name`).
template <typename Derived, typename RosMsg>
class TypedTopicMapper : public TopicMapper {
 public:
  bool supports_lazy() const override { return true; }

  rclcpp::SubscriptionBase::SharedPtr create_ros2_to_bus_subscription(
      rclcpp::Node::SharedPtr ros_node, const std::string &ros_topic,
      std::shared_ptr<TopicPublisher> bus_pub, std::shared_ptr<std::mutex> mtx,
      const rclcpp::QoS &qos) override {
    auto *self = static_cast<Derived *>(this);
    return ros_node->template create_subscription<RosMsg>(
        ros_topic, qos,
        [self, bus_pub, mtx](typename RosMsg::ConstSharedPtr msg) {
          try {
            auto bytes = self->ros_to_bus(*msg);
            std::lock_guard<std::mutex> lock(*mtx);
            bus_pub->publish(bytes);
          } catch (...) {
          }
        });
  }

  void attach(TopicWireContext &ctx) override {
    if (ctx.direction == Direction::Ros2ToBus) {
      auto pub = std::make_shared<TopicPublisher>(
          ctx.bus_qos_depth > 0
          ? ctx.bus_node.create_publisher(ctx.bus_topic.c_str(), ctx.bus_qos_depth)
          : ctx.bus_node.create_publisher(ctx.bus_topic.c_str()));
      auto mtx = std::make_shared<std::mutex>();
      auto sub = create_ros2_to_bus_subscription(ctx.ros_node, ctx.ros_topic, pub, mtx, ctx.qos);
      ctx.retain(std::move(pub));
      ctx.retain(std::move(mtx));
      ctx.retain(std::move(sub));
    } else {
      auto *self = static_cast<Derived *>(this);
      auto ros_pub = ctx.ros_node->template create_publisher<RosMsg>(ctx.ros_topic, ctx.qos);
      auto weak_pub = std::weak_ptr<rclcpp::Publisher<RosMsg>>(ros_pub);
      ctx.retain(std::make_shared<SubscriptionHandle>(ctx.bus_node.create_subscription(
          ctx.bus_topic.c_str(),
          [self, weak_pub](std::string_view, BytesView payload) {
            auto pub = weak_pub.lock();
            if (!pub) {
              return;
            }
            try {
              pub->publish(self->bus_to_ros(payload));
            } catch (...) {
            }
          },
          nullptr, ctx.bus_qos_depth)));
      ctx.retain(std::move(ros_pub));
    }
  }
};

/// Service: Derived provides `ros_req_to_bus` / `bus_req_to_ros` /
/// `ros_resp_to_bus` / `bus_resp_to_ros` (+ optional `error_response`).
template <typename Derived, typename RosSrv>
class TypedServiceMapper : public ServiceMapper {
 public:
  using Request = typename RosSrv::Request;
  using Response = typename RosSrv::Response;

  void attach(ServiceWireContext &ctx) override {
    auto *self = static_cast<Derived *>(this);
    const double timeout = ctx.timeout_secs;
    if (ctx.direction == Direction::Ros2ToBus) {
      auto bus_client =
          std::make_shared<ServiceClient>(
              ctx.bus_node.create_client(ctx.bus_service.c_str(), ctx.bus_qos.depth()));
      auto mtx = std::make_shared<std::mutex>();
      auto srv = ctx.ros_node->template create_service<RosSrv>(
          ctx.ros_service,
          [self, bus_client, mtx, timeout](const std::shared_ptr<Request> request,
                                           std::shared_ptr<Response> response) {
            try {
              auto req_bytes = self->ros_req_to_bus(*request);
              std::vector<uint8_t> resp_bytes;
              {
                std::lock_guard<std::mutex> lock(*mtx);
                resp_bytes = bus_client->call(req_bytes, timeout);
              }
              *response = self->bus_resp_to_ros(resp_bytes);
            } catch (const std::exception &e) {
              *response = self->error_response(std::string("bus call failed: ") + e.what());
            } catch (...) {
              *response = self->error_response("bus call failed");
            }
          },
          service_rmw_qos(ctx.ros_qos), ctx.callback_group);
      ctx.retain(std::move(bus_client));
      ctx.retain(std::move(mtx));
      ctx.retain(std::move(srv));
    } else {
      auto ros_client = ctx.ros_node->template create_client<RosSrv>(
          ctx.ros_service, service_rmw_qos(ctx.ros_qos), ctx.callback_group);
      ctx.retain(ros_client);
      ctx.retain(std::make_shared<ServiceHandle>(ctx.bus_node.create_service(
          ctx.bus_service.c_str(),
          [self, ros_client, timeout](BytesView body) -> std::vector<uint8_t> {
            if (!ros_client->wait_for_service(std::chrono::duration<double>(timeout))) {
              return self->ros_resp_to_bus(
                  self->error_response("timed out waiting for ROS service"));
            }
            auto req = std::make_shared<Request>(self->bus_req_to_ros(body));
            auto future = ros_client->async_send_request(req);
            const auto status = future.wait_for(std::chrono::duration<double>(timeout));
            if (status != std::future_status::ready) {
              return self->ros_resp_to_bus(
                  self->error_response("timed out waiting for ROS response"));
            }
            return self->ros_resp_to_bus(*future.get());
          },
          nullptr, ctx.bus_qos.depth()));
    }
  }

 protected:
  /// Override for richer failure responses (default: default-constructed Response).
  Response error_response(const std::string & /*message*/) const { return Response{}; }
};

/// Action (Ros2ToBus + BusToRos2): Derived provides goal/feedback/result converts.
template <typename Derived, typename RosAction>
class TypedActionMapper : public ActionMapper {
 public:
  using Goal = typename RosAction::Goal;
  using Feedback = typename RosAction::Feedback;
  using Result = typename RosAction::Result;
  using GoalHandle = rclcpp_action::ServerGoalHandle<RosAction>;

  void attach(ActionWireContext &ctx) override {
    auto *self = static_cast<Derived *>(this);
    const double timeout = ctx.timeout_secs;
    if (ctx.direction == Direction::Ros2ToBus) {
      auto bus_client = std::make_shared<ActionClient>(
          ctx.bus_node.create_action_client(ctx.bus_action.c_str(), ctx.bus_qos.depth()));
      auto mtx = std::make_shared<std::mutex>();

      auto handle_goal = [](const rclcpp_action::GoalUUID &, std::shared_ptr<const Goal>) {
        return rclcpp_action::GoalResponse::ACCEPT_AND_EXECUTE;
      };
      auto handle_cancel = [](const std::shared_ptr<GoalHandle>) {
        return rclcpp_action::CancelResponse::ACCEPT;
      };
      auto handle_accepted = [self, bus_client, mtx, timeout](
                                 const std::shared_ptr<GoalHandle> goal_handle) {
        std::thread([self, bus_client, mtx, timeout, goal_handle]() {
          const auto goal = goal_handle->get_goal();
          try {
            auto goal_bytes = self->ros_goal_to_bus(*goal);
            ActionGoalHandle handle = [&]() {
              std::lock_guard<std::mutex> lock(*mtx);
              return bus_client->send_goal(
                  goal_bytes,
                  [self, goal_handle](const ActionMessage &message) {
                    if (message.kind != "FEEDBACK") {
                      return;
                    }
                    try {
                      auto feedback = std::make_shared<Feedback>(
                          self->bus_feedback_to_ros(message.body));
                      goal_handle->publish_feedback(feedback);
                    } catch (...) {
                    }
                  },
                  nullptr, timeout);
            }();
            auto result_msg = handle.wait_result(timeout);
            if (result_msg.kind != "RESULT") {
              goal_handle->abort(std::make_shared<Result>());
              return;
            }
            auto result = std::make_shared<Result>(self->bus_result_to_ros(result_msg.body));
            goal_handle->succeed(result);
          } catch (...) {
            goal_handle->abort(std::make_shared<Result>());
          }
        }).detach();
      };

      auto server = rclcpp_action::create_server<RosAction>(
          ctx.ros_node, ctx.ros_action, handle_goal, handle_cancel, handle_accepted,
          action_server_qos(ctx.ros_qos), ctx.callback_group);
      ctx.retain(std::move(bus_client));
      ctx.retain(std::move(mtx));
      ctx.retain(std::move(server));
    } else {
      auto ros_client = rclcpp_action::create_client<RosAction>(
          ctx.ros_node, ctx.ros_action, ctx.callback_group, action_client_qos(ctx.ros_qos));
      auto mtx = std::make_shared<std::mutex>();
      ctx.retain(ros_client);
      ctx.retain(mtx);
      ctx.retain(std::make_shared<ActionServerHandle>(ctx.bus_node.create_action_server(
          ctx.bus_action.c_str(),
          [self, ros_client, mtx, timeout](BytesView body)
              -> std::vector<std::pair<std::string, std::vector<uint8_t>>> {
            Goal goal;
            try {
              goal = self->bus_goal_to_ros(body);
            } catch (...) {
              return {{"RESULT", self->ros_result_to_bus(Result{})}};
            }
            if (!ros_client->wait_for_action_server(std::chrono::duration<double>(timeout))) {
              return {{"RESULT", self->ros_result_to_bus(Result{})}};
            }
            auto feedbacks = std::make_shared<std::vector<std::vector<uint8_t>>>();
            auto feedback_mtx = std::make_shared<std::mutex>();
            typename rclcpp_action::Client<RosAction>::SendGoalOptions opts;
            opts.feedback_callback =
                [self, feedbacks, feedback_mtx](
                    typename rclcpp_action::ClientGoalHandle<RosAction>::SharedPtr,
                    const std::shared_ptr<const Feedback> feedback) {
                  try {
                    auto bytes = self->ros_feedback_to_bus(*feedback);
                    std::lock_guard<std::mutex> lock(*feedback_mtx);
                    feedbacks->push_back(std::move(bytes));
                  } catch (...) {
                  }
                };
            std::shared_future<typename rclcpp_action::ClientGoalHandle<RosAction>::SharedPtr>
                goal_future;
            {
              std::lock_guard<std::mutex> lock(*mtx);
              goal_future = ros_client->async_send_goal(goal, opts);
            }
            if (goal_future.wait_for(std::chrono::duration<double>(timeout)) !=
                std::future_status::ready) {
              return {{"RESULT", self->ros_result_to_bus(Result{})}};
            }
            auto goal_handle = goal_future.get();
            if (!goal_handle) {
              return {{"RESULT", self->ros_result_to_bus(Result{})}};
            }
            auto result_future = ros_client->async_get_result(goal_handle);
            if (result_future.wait_for(std::chrono::duration<double>(timeout)) !=
                std::future_status::ready) {
              return {{"RESULT", self->ros_result_to_bus(Result{})}};
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
                phases.emplace_back("RESULT", self->ros_result_to_bus(*wrapped.result));
              } else {
                phases.emplace_back("RESULT", self->ros_result_to_bus(Result{}));
              }
            } catch (...) {
              return {{"RESULT", self->ros_result_to_bus(Result{})}};
            }
            return phases;
          },
          nullptr, ctx.bus_qos.depth()));
    }
  }
};

}  // namespace robot_bus
