#pragma once

#include <robot_bus.h>

#include <cstdint>
#include <cstring>
#include <functional>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace robot_bus {

/// Non-owning view of bytes (C++17; avoids requiring C++20 std::span).
struct BytesView {
  const uint8_t *data = nullptr;
  size_t size = 0;

  BytesView() = default;
  BytesView(const uint8_t *d, size_t n) : data(d), size(n) {}
  BytesView(const std::vector<uint8_t> &v) : data(v.data()), size(v.size()) {}
  BytesView(const std::string &s)
      : data(reinterpret_cast<const uint8_t *>(s.data())), size(s.size()) {}
  BytesView(std::string_view s)
      : data(reinterpret_cast<const uint8_t *>(s.data())), size(s.size()) {}
};

inline std::string last_error() {
  const char *e = robot_bus_last_error();
  return e ? std::string(e) : std::string();
}

class Error : public std::runtime_error {
 public:
  explicit Error(std::string msg) : std::runtime_error(std::move(msg)) {}
};

inline void check(int rc, const char *what) {
  if (rc < 0) {
    auto err = last_error();
    throw Error(std::string(what) + ": " + (err.empty() ? "unknown error" : err));
  }
}

inline void *check_ptr(void *p, const char *what) {
  if (!p) {
    auto err = last_error();
    throw Error(std::string(what) + ": " + (err.empty() ? "null" : err));
  }
  return p;
}

/// Local node parameter value (bool / int64 / double / string).
using ParameterValue = std::variant<bool, int64_t, double, std::string>;

struct Parameter {
  std::string name;
  ParameterValue value;
};

inline ParameterValue parameter_value_from_c(RobotBusParameterValue &v) {
  ParameterValue out;
  switch (v.type) {
    case ROBOT_BUS_PARAM_BOOL:
      out = v.bool_value != 0;
      break;
    case ROBOT_BUS_PARAM_INTEGER:
      out = v.integer_value;
      break;
    case ROBOT_BUS_PARAM_DOUBLE:
      out = v.double_value;
      break;
    case ROBOT_BUS_PARAM_STRING:
      out = v.string_value ? std::string(v.string_value) : std::string();
      robot_bus_free_string(v.string_value);
      v.string_value = nullptr;
      break;
    default:
      throw Error("unknown parameter type");
  }
  return out;
}

/// Shared ZeroMQ runtime context (required for same-process inproc).
class Context {
 public:
  Context() {
    c_ = static_cast<RobotBusContext *>(check_ptr(robot_bus_context_new(), "Context"));
  }

  explicit Context(RobotBusContext *raw) : c_(raw) {}

  ~Context() { robot_bus_context_free(c_); }

  Context(const Context &o) {
    c_ = static_cast<RobotBusContext *>(
        check_ptr(robot_bus_context_clone(o.c_), "Context::clone"));
  }

  Context &operator=(const Context &o) {
    if (this != &o) {
      RobotBusContext *next = static_cast<RobotBusContext *>(
          check_ptr(robot_bus_context_clone(o.c_), "Context::clone"));
      robot_bus_context_free(c_);
      c_ = next;
    }
    return *this;
  }

  Context(Context &&o) noexcept : c_(o.c_) { o.c_ = nullptr; }

  Context &operator=(Context &&o) noexcept {
    if (this != &o) {
      robot_bus_context_free(c_);
      c_ = o.c_;
      o.c_ = nullptr;
    }
    return *this;
  }

  RobotBusContext *raw() { return c_; }
  const RobotBusContext *raw() const { return c_; }

 private:
  RobotBusContext *c_ = nullptr;
};

inline uint8_t *alloc_reply_bytes(BytesView payload) {
  if (payload.size == 0) {
    return nullptr;
  }
  uint8_t *buf = robot_bus_alloc_bytes(payload.size);
  if (!buf) {
    throw Error("robot_bus_alloc_bytes failed");
  }
  std::memcpy(buf, payload.data, payload.size);
  return buf;
}

class OwnedString {
 public:
  explicit OwnedString(char *p) : p_(p) {}
  ~OwnedString() { robot_bus_free_string(p_); }
  OwnedString(const OwnedString &) = delete;
  OwnedString &operator=(const OwnedString &) = delete;
  OwnedString(OwnedString &&o) noexcept : p_(o.p_) { o.p_ = nullptr; }
  std::string str() const { return p_ ? std::string(p_) : std::string(); }

 private:
  char *p_;
};

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

class Node {
 public:
  using MsgCallback = std::function<void(std::string_view topic, BytesView payload)>;
  using TimerCallback = std::function<void()>;
  using ServiceHandler = std::function<std::vector<uint8_t>(BytesView body)>;
  using ActionHandler =
      std::function<std::vector<std::pair<std::string, std::vector<uint8_t>>>(BytesView body)>;

  explicit Node(std::string name) {
    n_ = static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_new(name.c_str(), nullptr), "Node"));
  }

  /// `opts` pointers must remain valid for the duration of this call.
  Node(std::string name, const RobotBusNodeOptions &opts) {
    n_ = static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_new(name.c_str(), &opts), "Node"));
  }

  Node(RobotBusNode *raw) : n_(raw) {}

  static Node tcp(std::string name, const char *host = "localhost") {
    return Node(static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_tcp(name.c_str(), host), "Node::tcp")));
  }

  static Node ipc(std::string name, const char *path = nullptr) {
    return Node(static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_ipc(name.c_str(), path), "Node::ipc")));
  }

  static Node inproc(std::string name, const char *prefix = nullptr) {
    return Node(static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_inproc(name.c_str(), prefix), "Node::inproc")));
  }

  /// Same-process inproc sharing `ctx` with an embedded broker.
  static Node inproc_with_context(Context &ctx, std::string name,
                                  const char *prefix = nullptr) {
    return Node(static_cast<RobotBusNode *>(check_ptr(
        robot_bus_node_inproc_with_context(ctx.raw(), name.c_str(), prefix),
        "Node::inproc_with_context")));
  }

  /// ROS 2–style preferred entry: shared context + local TCP.
  static Node with_context(Context &ctx, std::string name,
                           const RobotBusNodeOptions *opts = nullptr) {
    return Node(static_cast<RobotBusNode *>(check_ptr(
        robot_bus_node_new_with_context(ctx.raw(), name.c_str(), opts),
        "Node::with_context")));
  }

  static Node ws(std::string name) {
    return Node(static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_ws(name.c_str()), "Node::ws")));
  }

  static Node ws_at(std::string name, const char *url) {
    return Node(static_cast<RobotBusNode *>(
        check_ptr(robot_bus_node_ws_at(name.c_str(), url), "Node::ws_at")));
  }

  /// Discover a broker (HTTP /api/v1/discover) then connect with the chosen transport.
  static Node discover(std::string name, const char *transport = "tcp",
                       const RobotBusDiscoverOpts *opts = nullptr) {
    return Node(static_cast<RobotBusNode *>(check_ptr(
        robot_bus_node_discover(name.c_str(), transport, opts), "Node::discover")));
  }

  ~Node() { robot_bus_node_free(n_); }
  Node(const Node &) = delete;
  Node &operator=(const Node &) = delete;
  Node(Node &&o) noexcept
      : n_(o.n_),
        msg_cbs_(std::move(o.msg_cbs_)),
        timer_cbs_(std::move(o.timer_cbs_)),
        svc_cbs_(std::move(o.svc_cbs_)),
        action_cbs_(std::move(o.action_cbs_)) {
    o.n_ = nullptr;
  }

  std::string name() const {
    OwnedString s(robot_bus_node_name(n_));
    return s.str();
  }

  [[nodiscard]] TopicPublisher create_publisher(const char *topic) {
    return TopicPublisher(static_cast<RobotBusTopicPublisher *>(
        check_ptr(robot_bus_node_create_publisher(n_, topic), "create_publisher")));
  }

  [[nodiscard]] TopicPublisher create_publisher(const char *topic, int32_t qos_depth) {
    return TopicPublisher(static_cast<RobotBusTopicPublisher *>(check_ptr(
        robot_bus_node_create_publisher_with_qos(n_, topic, qos_depth),
        "create_publisher_with_qos")));
  }

  /// Keep the returned handle; dropping it unsubscribes (ROS 2 shared_ptr style).
  [[nodiscard]] SubscriptionHandle create_subscription(const char *topic, MsgCallback cb,
                                                       const CallbackGroup *group = nullptr) {
    return create_subscription(topic, std::move(cb), group, 0);
  }

  [[nodiscard]] SubscriptionHandle create_subscription(const char *topic, MsgCallback cb,
                                                       const CallbackGroup *group,
                                                       int32_t qos_depth) {
    msg_cbs_.push_back(std::make_unique<MsgCallback>(std::move(cb)));
    MsgCallback *held = msg_cbs_.back().get();
    return SubscriptionHandle(
        n_, static_cast<RobotBusSubscriptionHandle *>(check_ptr(
                robot_bus_node_create_subscription_with_qos(
                    n_, topic,
                    [](const char *t, const uint8_t *data, size_t len, void *user) {
                      auto *fn = static_cast<MsgCallback *>(user);
                      (*fn)(t ? std::string_view(t) : std::string_view(), BytesView(data, len));
                    },
                    held, group ? group->raw() : nullptr, qos_depth),
                "create_subscription")));
  }

  std::optional<std::vector<uint8_t>> wait_for_message(const char *topic,
                                                        double timeout_secs = -1.0) {
    uint8_t *out = nullptr;
    size_t len = 0;
    int rc = robot_bus_node_wait_for_message(n_, topic, timeout_secs, &out, &len);
    if (rc < 0) {
      check(rc, "wait_for_message");
    }
    if (rc == 0) {
      return std::nullopt;
    }
    std::vector<uint8_t> result(out, out + len);
    robot_bus_free_bytes(out, len);
    return result;
  }

  [[nodiscard]] TimerHandle create_timer(double period_secs, TimerCallback cb,
                                         const CallbackGroup *group = nullptr) {
    timer_cbs_.push_back(std::make_unique<TimerCallback>(std::move(cb)));
    TimerCallback *held = timer_cbs_.back().get();
    return TimerHandle(static_cast<RobotBusTimerHandle *>(check_ptr(
        robot_bus_node_create_timer(
            n_, period_secs, [](void *user) { (*static_cast<TimerCallback *>(user))(); }, held,
            group ? group->raw() : nullptr),
        "create_timer")));
  }

  /// Alias for create_timer (ROS 2 `create_wall_timer`).
  [[nodiscard]] TimerHandle create_wall_timer(double period_secs, TimerCallback cb,
                                              const CallbackGroup *group = nullptr) {
    return create_timer(period_secs, std::move(cb), group);
  }

  void cancel_timer(const TimerHandle &handle) {
    check(robot_bus_node_cancel_timer(n_, handle.raw()), "cancel_timer");
  }

  /// Keep the returned handle; dropping it destroys the service.
  [[nodiscard]] ServiceHandle create_service(const char *service_name, ServiceHandler handler,
                                             const CallbackGroup *group = nullptr) {
    svc_cbs_.push_back(std::make_unique<ServiceHandler>(std::move(handler)));
    ServiceHandler *held = svc_cbs_.back().get();
    return ServiceHandle(
        n_, static_cast<RobotBusServiceHandle *>(check_ptr(
                robot_bus_node_create_service(
                    n_, service_name,
                    [](const uint8_t *data, size_t len, uint8_t **out_data, size_t *out_len,
                       void *user) -> int {
                      try {
                        auto *fn = static_cast<ServiceHandler *>(user);
                        auto reply = (*fn)(BytesView(data, len));
                        *out_len = reply.size();
                        *out_data = alloc_reply_bytes(reply);
                        return 0;
                      } catch (...) {
                        *out_data = nullptr;
                        *out_len = 0;
                        return -1;
                      }
                    },
                    held, group ? group->raw() : nullptr),
                "create_service")));
  }

  ServiceClient create_client(const char *service_name) {
    return ServiceClient(static_cast<RobotBusServiceClient *>(
        check_ptr(robot_bus_node_create_client(n_, service_name), "create_client")));
  }

  /// Keep the returned handle; dropping it destroys the action server.
  [[nodiscard]] ActionServerHandle create_action_server(const char *action_name,
                                                        ActionHandler handler,
                                                        const CallbackGroup *group = nullptr) {
    action_cbs_.push_back(std::make_unique<ActionHandler>(std::move(handler)));
    ActionHandler *held = action_cbs_.back().get();
    return ActionServerHandle(
        n_, static_cast<RobotBusActionServerHandle *>(check_ptr(
                robot_bus_node_create_action_server(
                    n_, action_name,
                    [](const uint8_t *data, size_t len, RobotBusActionPhase **out_phases,
                       size_t *out_count, void *user) -> int {
                      try {
                        auto *fn = static_cast<ActionHandler *>(user);
                        auto phases = (*fn)(BytesView(data, len));
                        *out_count = phases.size();
                        if (phases.empty()) {
                          *out_phases = nullptr;
                          return 0;
                        }
                        RobotBusActionPhase *arr = robot_bus_alloc_action_phases(phases.size());
                        if (!arr) {
                          *out_phases = nullptr;
                          *out_count = 0;
                          return -1;
                        }
                        for (size_t i = 0; i < phases.size(); ++i) {
                          arr[i].phase = robot_bus_dup_string(phases[i].first.c_str());
                          arr[i].body_len = phases[i].second.size();
                          arr[i].body = alloc_reply_bytes(phases[i].second);
                        }
                        *out_phases = arr;
                        return 0;
                      } catch (...) {
                        *out_phases = nullptr;
                        *out_count = 0;
                        return -1;
                      }
                    },
                    held, group ? group->raw() : nullptr),
                "create_action_server")));
  }

  ActionClient create_action_client(const char *action_name) {
    return ActionClient(static_cast<RobotBusActionClient *>(check_ptr(
        robot_bus_node_create_action_client(n_, action_name), "create_action_client")));
  }

  void connect_action_client() {
    check(robot_bus_node_connect_action_client(n_), "connect_action_client");
  }

  CallbackGroup create_callback_group(int kind = 0) {
    return CallbackGroup(static_cast<RobotBusCallbackGroup *>(
        check_ptr(robot_bus_node_create_callback_group(n_, kind), "create_callback_group")));
  }

  ShutdownHandle shutdown_handle() {
    return ShutdownHandle(static_cast<RobotBusShutdownHandle *>(
        check_ptr(robot_bus_node_shutdown_handle(n_), "shutdown_handle")));
  }

  void shutdown() { check(robot_bus_node_shutdown(n_), "shutdown"); }

  bool spin_once(double timeout_secs = -1.0) {
    int rc = robot_bus_node_spin_once(n_, timeout_secs);
    if (rc < 0) {
      check(rc, "spin_once");
    }
    return rc == 1;
  }

  void spin() { check(robot_bus_node_spin(n_), "spin"); }
  void start() { check(robot_bus_node_start(n_), "start"); }
  void stop() { check(robot_bus_node_stop(n_), "stop"); }
  void wait() { check(robot_bus_node_wait(n_), "wait"); }

  void declare_parameter(const char *name, bool value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_BOOL;
    v.bool_value = value ? 1 : 0;
    check(robot_bus_node_declare_parameter(n_, name, &v), "declare_parameter");
  }
  void declare_parameter(const char *name, int64_t value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_INTEGER;
    v.integer_value = value;
    check(robot_bus_node_declare_parameter(n_, name, &v), "declare_parameter");
  }
  void declare_parameter(const char *name, double value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_DOUBLE;
    v.double_value = value;
    check(robot_bus_node_declare_parameter(n_, name, &v), "declare_parameter");
  }
  void declare_parameter(const char *name, const char *value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_STRING;
    v.string_value = const_cast<char *>(value);
    check(robot_bus_node_declare_parameter(n_, name, &v), "declare_parameter");
  }
  void declare_parameter(const char *name, const std::string &value) {
    declare_parameter(name, value.c_str());
  }

  void set_parameter(const char *name, bool value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_BOOL;
    v.bool_value = value ? 1 : 0;
    check(robot_bus_node_set_parameter(n_, name, &v), "set_parameter");
  }
  void set_parameter(const char *name, int64_t value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_INTEGER;
    v.integer_value = value;
    check(robot_bus_node_set_parameter(n_, name, &v), "set_parameter");
  }
  void set_parameter(const char *name, double value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_DOUBLE;
    v.double_value = value;
    check(robot_bus_node_set_parameter(n_, name, &v), "set_parameter");
  }
  void set_parameter(const char *name, const char *value) {
    RobotBusParameterValue v{};
    v.type = ROBOT_BUS_PARAM_STRING;
    v.string_value = const_cast<char *>(value);
    check(robot_bus_node_set_parameter(n_, name, &v), "set_parameter");
  }
  void set_parameter(const char *name, const std::string &value) {
    set_parameter(name, value.c_str());
  }

  RobotBusParameterValue get_parameter_raw(const char *name) {
    RobotBusParameterValue out{};
    check(robot_bus_node_get_parameter(n_, name, &out), "get_parameter");
    return out;
  }

  ParameterValue get_parameter(const char *name) {
    RobotBusParameterValue raw = get_parameter_raw(name);
    return parameter_value_from_c(raw);
  }

  bool has_parameter(const char *name) const {
    int rc = robot_bus_node_has_parameter(n_, name);
    if (rc < 0) {
      check(rc, "has_parameter");
    }
    return rc == 1;
  }

  void undeclare_parameter(const char *name) {
    check(robot_bus_node_undeclare_parameter(n_, name), "undeclare_parameter");
  }

  struct ListParametersResult {
    std::vector<std::string> names;
    std::vector<std::string> prefixes;
  };

  /// ROS-shaped list (`depth == 0` = recursive).
  ListParametersResult list_parameters(const std::vector<std::string> &prefixes = {},
                                       uint64_t depth = 0) {
    std::vector<const char *> prefix_ptrs;
    prefix_ptrs.reserve(prefixes.size());
    for (const auto &p : prefixes) {
      prefix_ptrs.push_back(p.c_str());
    }
    char **out_names = nullptr;
    size_t names_count = 0;
    char **out_prefixes = nullptr;
    size_t prefixes_count = 0;
    check(robot_bus_node_list_parameters(n_, prefix_ptrs.empty() ? nullptr : prefix_ptrs.data(),
                                        prefix_ptrs.size(), depth, &out_names, &names_count,
                                        &out_prefixes, &prefixes_count),
          "list_parameters");
    ListParametersResult result;
    result.names.reserve(names_count);
    for (size_t i = 0; i < names_count; ++i) {
      result.names.emplace_back(out_names[i] ? out_names[i] : "");
    }
    result.prefixes.reserve(prefixes_count);
    for (size_t i = 0; i < prefixes_count; ++i) {
      result.prefixes.emplace_back(out_prefixes[i] ? out_prefixes[i] : "");
    }
    robot_bus_string_list_free(out_names, names_count);
    robot_bus_string_list_free(out_prefixes, prefixes_count);
    return result;
  }

  std::vector<Parameter> list_all_parameters() {
    RobotBusParameter *raw = nullptr;
    size_t count = 0;
    check(robot_bus_node_list_all_parameters(n_, &raw, &count), "list_all_parameters");
    std::vector<Parameter> out;
    out.reserve(count);
    for (size_t i = 0; i < count; ++i) {
      Parameter p;
      p.name = raw[i].name ? std::string(raw[i].name) : std::string();
      p.value = parameter_value_from_c(raw[i].value);
      out.push_back(std::move(p));
    }
    robot_bus_parameters_free(raw, count);
    return out;
  }

  void load_parameters_from_yaml(const char *path) {
    check(robot_bus_node_load_parameters_from_yaml(n_, path), "load_parameters_from_yaml");
  }

  void load_parameters_from_yaml_str(const char *yaml) {
    check(robot_bus_node_load_parameters_from_yaml_str(n_, yaml),
          "load_parameters_from_yaml_str");
  }

  RobotBusNode *raw() { return n_; }

 private:
  RobotBusNode *n_ = nullptr;
  std::vector<std::unique_ptr<MsgCallback>> msg_cbs_;
  std::vector<std::unique_ptr<TimerCallback>> timer_cbs_;
  std::vector<std::unique_ptr<ServiceHandler>> svc_cbs_;
  std::vector<std::unique_ptr<ActionHandler>> action_cbs_;
};

class Broker {
 public:
  Broker() {
    b_ = static_cast<RobotBusBroker *>(check_ptr(robot_bus_broker_start(nullptr), "Broker"));
  }

  explicit Broker(const RobotBusBrokerOptions &opts) {
    b_ = static_cast<RobotBusBroker *>(check_ptr(robot_bus_broker_start(&opts), "Broker"));
  }

  /// Start broker sharing `ctx` (required for same-process inproc Nodes).
  explicit Broker(Context &ctx) {
    b_ = static_cast<RobotBusBroker *>(
        check_ptr(robot_bus_broker_start_with_context(ctx.raw(), nullptr), "Broker"));
  }

  Broker(Context &ctx, const RobotBusBrokerOptions &opts) {
    b_ = static_cast<RobotBusBroker *>(
        check_ptr(robot_bus_broker_start_with_context(ctx.raw(), &opts), "Broker"));
  }

  ~Broker() { robot_bus_broker_free(b_); }
  Broker(const Broker &) = delete;
  Broker &operator=(const Broker &) = delete;
  Broker(Broker &&o) noexcept : b_(o.b_) { o.b_ = nullptr; }

  void stop() { check(robot_bus_broker_stop(b_), "broker stop"); }

  std::string message_xsub_bind() const {
    OwnedString s(robot_bus_broker_message_xsub_bind(b_));
    return s.str();
  }
  std::string message_xpub_bind() const {
    OwnedString s(robot_bus_broker_message_xpub_bind(b_));
    return s.str();
  }
  std::string service_frontend_bind() const {
    OwnedString s(robot_bus_broker_service_frontend_bind(b_));
    return s.str();
  }
  std::string service_backend_bind() const {
    OwnedString s(robot_bus_broker_service_backend_bind(b_));
    return s.str();
  }
  std::string action_frontend_bind() const {
    OwnedString s(robot_bus_broker_action_frontend_bind(b_));
    return s.str();
  }
  std::string action_backend_bind() const {
    OwnedString s(robot_bus_broker_action_backend_bind(b_));
    return s.str();
  }
  std::string api_listen() const {
    OwnedString s(robot_bus_broker_api_listen(b_));
    return s.str();
  }

 private:
  RobotBusBroker *b_ = nullptr;
};

}  // namespace robot_bus
