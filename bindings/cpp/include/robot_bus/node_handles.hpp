#pragma once

#include <robot_bus/node_core.hpp>

namespace robot_bus {

class TopicPublisher {
 public:
  explicit TopicPublisher(RobotBusTopicPublisher *p) : p_(p) {}
  ~TopicPublisher() { robot_bus_topic_publisher_free(p_); }
  TopicPublisher(const TopicPublisher &) = delete;
  TopicPublisher &operator=(const TopicPublisher &) = delete;
  TopicPublisher(TopicPublisher &&o) noexcept : p_(o.p_) { o.p_ = nullptr; }

  std::string topic() const {
    OwnedString s(robot_bus_topic_publisher_topic(p_));
    return s.str();
  }

  void publish(BytesView payload) {
    check(robot_bus_topic_publisher_publish(p_, payload.data, payload.size), "publish");
  }

 private:
  RobotBusTopicPublisher *p_;
};

class ServiceClient {
 public:
  explicit ServiceClient(RobotBusServiceClient *c) : c_(c) {}
  ~ServiceClient() { robot_bus_service_client_free(c_); }
  ServiceClient(const ServiceClient &) = delete;
  ServiceClient &operator=(const ServiceClient &) = delete;
  ServiceClient(ServiceClient &&o) noexcept : c_(o.c_) { o.c_ = nullptr; }

  std::string service_name() const {
    OwnedString s(robot_bus_service_client_service_name(c_));
    return s.str();
  }

  std::vector<uint8_t> call(BytesView body, double timeout_secs = -1.0) {
    uint8_t *out = nullptr;
    size_t len = 0;
    check(robot_bus_service_client_call(c_, body.data, body.size, timeout_secs, &out, &len),
          "service call");
    std::vector<uint8_t> result(out, out + len);
    robot_bus_free_bytes(out, len);
    return result;
  }

  bool service_is_ready() const {
    return robot_bus_service_client_service_is_ready(c_) != 0;
  }

  bool wait_for_service(double timeout_secs = -1.0) const {
    return robot_bus_service_client_wait_for_service(c_, timeout_secs) != 0;
  }

 private:
  RobotBusServiceClient *c_;
};

struct ActionMessage {
  std::string kind;
  std::vector<uint8_t> body;
  std::string goal_id;
  std::string action_name;
};

inline ActionMessage copy_action_message(const RobotBusActionMessage &message) {
  ActionMessage out;
  out.kind = message.kind ? message.kind : "";
  if (message.body && message.body_len != 0) {
    out.body.assign(message.body, message.body + message.body_len);
  }
  out.goal_id = message.goal_id ? message.goal_id : "";
  out.action_name = message.action_name ? message.action_name : "";
  return out;
}

inline void free_action_message_fields(RobotBusActionMessage &message) {
  robot_bus_free_string(message.kind);
  robot_bus_free_bytes(message.body, message.body_len);
  robot_bus_free_string(message.goal_id);
  robot_bus_free_string(message.action_name);
  message = {};
}

using ActionFeedbackCallback = std::function<void(const ActionMessage &)>;

struct ActionFeedbackState {
  explicit ActionFeedbackState(ActionFeedbackCallback callback)
      : callback(std::move(callback)) {}

  ActionFeedbackCallback callback;
};

class ActionGoalHandle {
 public:
  ActionGoalHandle(RobotBusActionGoalHandle *handle,
                   std::shared_ptr<ActionFeedbackState> feedback_state)
      : handle_(handle), feedback_state_(std::move(feedback_state)) {}

  ~ActionGoalHandle() { reset(); }
  ActionGoalHandle(const ActionGoalHandle &) = delete;
  ActionGoalHandle &operator=(const ActionGoalHandle &) = delete;

  ActionGoalHandle(ActionGoalHandle &&other) noexcept
      : handle_(other.handle_), feedback_state_(std::move(other.feedback_state_)) {
    other.handle_ = nullptr;
  }

  ActionGoalHandle &operator=(ActionGoalHandle &&other) noexcept {
    if (this != &other) {
      reset();
      handle_ = other.handle_;
      feedback_state_ = std::move(other.feedback_state_);
      other.handle_ = nullptr;
    }
    return *this;
  }

  std::string goal_id() const {
    OwnedString value(robot_bus_action_goal_handle_goal_id(handle_));
    return value.str();
  }

  std::string action_name() const {
    OwnedString value(robot_bus_action_goal_handle_action_name(handle_));
    return value.str();
  }

  ActionMessage wait_result(double timeout_secs = -1.0) {
    RobotBusActionMessage message{};
    check(robot_bus_action_goal_handle_wait_result(handle_, timeout_secs, &message),
          "wait_result");
    try {
      ActionMessage out = copy_action_message(message);
      free_action_message_fields(message);
      return out;
    } catch (...) {
      free_action_message_fields(message);
      throw;
    }
  }

  void cancel() {
    check(robot_bus_action_goal_handle_cancel(handle_), "cancel");
  }

 private:
  void reset() noexcept {
    if (handle_) {
      robot_bus_action_goal_handle_free(handle_);
      handle_ = nullptr;
    }
    feedback_state_.reset();
  }

  RobotBusActionGoalHandle *handle_ = nullptr;
  std::shared_ptr<ActionFeedbackState> feedback_state_;
};

class ActionClient {
 public:
  explicit ActionClient(RobotBusActionClient *c) : c_(c) {}
  ~ActionClient() { robot_bus_action_client_free(c_); }
  ActionClient(const ActionClient &) = delete;
  ActionClient &operator=(const ActionClient &) = delete;
  ActionClient(ActionClient &&o) noexcept : c_(o.c_) { o.c_ = nullptr; }

  std::string action_name() const {
    OwnedString s(robot_bus_action_client_action_name(c_));
    return s.str();
  }

  bool action_server_is_ready() const {
    return robot_bus_action_client_action_server_is_ready(c_) != 0;
  }

  bool wait_for_action_server(double timeout_secs = -1.0) const {
    return robot_bus_action_client_wait_for_action_server(c_, timeout_secs) != 0;
  }

  ActionGoalHandle send_goal(BytesView body, ActionFeedbackCallback feedback = {},
                             const char *goal_id = nullptr, double timeout_secs = -1.0) {
    auto state = std::make_shared<ActionFeedbackState>(std::move(feedback));
    RobotBusActionGoalHandle *handle = nullptr;
    check(robot_bus_action_client_send_goal(
              c_, body.data, body.size, goal_id, timeout_secs,
              [](const RobotBusActionMessage *message, void *user) {
                if (!message || !user) {
                  return;
                }
                auto *state = static_cast<ActionFeedbackState *>(user);
                if (!state->callback) {
                  return;
                }
                try {
                  const ActionMessage copied = copy_action_message(*message);
                  state->callback(copied);
                } catch (...) {
                  // Exceptions must not cross the C callback boundary.
                }
              },
              state.get(), &handle),
          "send_goal");
    return ActionGoalHandle(
        static_cast<RobotBusActionGoalHandle *>(check_ptr(handle, "send_goal handle")),
        std::move(state));
  }

  ActionGoalHandle send_goal(BytesView body, const char *goal_id, double timeout_secs,
                             ActionFeedbackCallback feedback = {}) {
    return send_goal(body, std::move(feedback), goal_id, timeout_secs);
  }

 private:
  RobotBusActionClient *c_;
};

class ShutdownHandle {
 public:
  explicit ShutdownHandle(RobotBusShutdownHandle *h) : h_(h) {}
  ~ShutdownHandle() { robot_bus_shutdown_handle_free(h_); }
  ShutdownHandle(const ShutdownHandle &) = delete;
  ShutdownHandle &operator=(const ShutdownHandle &) = delete;
  ShutdownHandle(ShutdownHandle &&o) noexcept : h_(o.h_) { o.h_ = nullptr; }

  void shutdown() { robot_bus_shutdown_handle_shutdown(h_); }
  bool is_running() const { return robot_bus_shutdown_handle_is_running(h_) != 0; }

 private:
  RobotBusShutdownHandle *h_;
};

class CallbackGroup {
 public:
  explicit CallbackGroup(RobotBusCallbackGroup *g) : g_(g) {}
  ~CallbackGroup() { robot_bus_callback_group_free(g_); }
  CallbackGroup(const CallbackGroup &) = delete;
  CallbackGroup &operator=(const CallbackGroup &) = delete;
  CallbackGroup(CallbackGroup &&o) noexcept : g_(o.g_) { o.g_ = nullptr; }

  uint64_t id() const { return robot_bus_callback_group_id(g_); }
  int kind() const { return robot_bus_callback_group_kind(g_); }
  const RobotBusCallbackGroup *raw() const { return g_; }

 private:
  RobotBusCallbackGroup *g_;
};

class TimerHandle {
 public:
  explicit TimerHandle(RobotBusTimerHandle *h) : h_(h) {}
  ~TimerHandle() { robot_bus_timer_handle_free(h_); }
  TimerHandle(const TimerHandle &) = delete;
  TimerHandle &operator=(const TimerHandle &) = delete;
  TimerHandle(TimerHandle &&o) noexcept : h_(o.h_) { o.h_ = nullptr; }
  const RobotBusTimerHandle *raw() const { return h_; }

 private:
  RobotBusTimerHandle *h_;
};

class SubscriptionHandle {
 public:
  SubscriptionHandle(RobotBusNode *n, RobotBusSubscriptionHandle *h) : n_(n), h_(h) {}
  ~SubscriptionHandle() {
    // Best-effort: destroy is rejected while start() is active; never throw.
    if (!destroyed_ && h_ && n_) {
      (void)robot_bus_node_destroy_subscription(n_, h_);
      destroyed_ = true;
    }
    robot_bus_subscription_handle_free(h_);
    h_ = nullptr;
  }
  SubscriptionHandle(const SubscriptionHandle &) = delete;
  SubscriptionHandle &operator=(const SubscriptionHandle &) = delete;
  SubscriptionHandle(SubscriptionHandle &&o) noexcept : n_(o.n_), h_(o.h_), destroyed_(o.destroyed_) {
    o.h_ = nullptr;
    o.destroyed_ = true;
  }
  void destroy() {
    if (destroyed_ || !h_ || !n_) {
      return;
    }
    check(robot_bus_node_destroy_subscription(n_, h_), "destroy_subscription");
    destroyed_ = true;
  }
  RobotBusSubscriptionHandle *raw() const { return h_; }

 private:
  RobotBusNode *n_ = nullptr;
  RobotBusSubscriptionHandle *h_ = nullptr;
  bool destroyed_ = false;
};

class ServiceHandle {
 public:
  ServiceHandle(RobotBusNode *n, RobotBusServiceHandle *h) : n_(n), h_(h) {}
  ~ServiceHandle() {
    if (!destroyed_ && h_ && n_) {
      (void)robot_bus_node_destroy_service(n_, h_);
      destroyed_ = true;
    }
    robot_bus_service_handle_free(h_);
    h_ = nullptr;
  }
  ServiceHandle(const ServiceHandle &) = delete;
  ServiceHandle &operator=(const ServiceHandle &) = delete;
  ServiceHandle(ServiceHandle &&o) noexcept : n_(o.n_), h_(o.h_), destroyed_(o.destroyed_) {
    o.h_ = nullptr;
    o.destroyed_ = true;
  }
  void destroy() {
    if (destroyed_ || !h_ || !n_) {
      return;
    }
    check(robot_bus_node_destroy_service(n_, h_), "destroy_service");
    destroyed_ = true;
  }
  std::string service_name() const {
    char *s = robot_bus_service_handle_name(h_);
    if (!s) {
      return {};
    }
    std::string out(s);
    robot_bus_free_string(s);
    return out;
  }
  RobotBusServiceHandle *raw() const { return h_; }

 private:
  RobotBusNode *n_ = nullptr;
  RobotBusServiceHandle *h_ = nullptr;
  bool destroyed_ = false;
};

class ActionServerHandle {
 public:
  ActionServerHandle(RobotBusNode *n, RobotBusActionServerHandle *h) : n_(n), h_(h) {}
  ~ActionServerHandle() {
    if (!destroyed_ && h_ && n_) {
      (void)robot_bus_node_destroy_action_server(n_, h_);
      destroyed_ = true;
    }
    robot_bus_action_server_handle_free(h_);
    h_ = nullptr;
  }
  ActionServerHandle(const ActionServerHandle &) = delete;
  ActionServerHandle &operator=(const ActionServerHandle &) = delete;
  ActionServerHandle(ActionServerHandle &&o) noexcept
      : n_(o.n_), h_(o.h_), destroyed_(o.destroyed_) {
    o.h_ = nullptr;
    o.destroyed_ = true;
  }
  void destroy() {
    if (destroyed_ || !h_ || !n_) {
      return;
    }
    check(robot_bus_node_destroy_action_server(n_, h_), "destroy_action_server");
    destroyed_ = true;
  }
  std::string action_name() const {
    char *s = robot_bus_action_server_handle_name(h_);
    if (!s) {
      return {};
    }
    std::string out(s);
    robot_bus_free_string(s);
    return out;
  }
  RobotBusActionServerHandle *raw() const { return h_; }

 private:
  RobotBusNode *n_ = nullptr;
  RobotBusActionServerHandle *h_ = nullptr;
  bool destroyed_ = false;
};

}  // namespace robot_bus
