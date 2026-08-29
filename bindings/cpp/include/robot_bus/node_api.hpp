#pragma once

#include <robot_bus/node_handles.hpp>

namespace robot_bus {

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

  std::string connection_state() const {
    OwnedString s(robot_bus_node_connection_state(n_));
    return s.str();
  }

  /// `timeout_secs < 0` waits until connected or shutdown.
  bool wait_for_broker(double timeout_secs = -1.0) const {
    return robot_bus_node_wait_for_broker(n_, timeout_secs) != 0;
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
    return create_service(service_name, std::move(handler), group, 0);
  }

  [[nodiscard]] ServiceHandle create_service(const char *service_name, ServiceHandler handler,
                                             const CallbackGroup *group, int32_t qos_depth) {
    svc_cbs_.push_back(std::make_unique<ServiceHandler>(std::move(handler)));
    ServiceHandler *held = svc_cbs_.back().get();
    return ServiceHandle(
        n_, static_cast<RobotBusServiceHandle *>(check_ptr(
                robot_bus_node_create_service_with_qos(
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
                    held, group ? group->raw() : nullptr, qos_depth),
                "create_service")));
  }

  ServiceClient create_client(const char *service_name) { return create_client(service_name, 0); }

  ServiceClient create_client(const char *service_name, int32_t qos_depth) {
    return ServiceClient(static_cast<RobotBusServiceClient *>(check_ptr(
        robot_bus_node_create_client_with_qos(n_, service_name, qos_depth), "create_client")));
  }

  /// Keep the returned handle; dropping it destroys the action server.
  [[nodiscard]] ActionServerHandle create_action_server(const char *action_name,
                                                        ActionHandler handler,
                                                        const CallbackGroup *group = nullptr) {
    return create_action_server(action_name, std::move(handler), group, 0);
  }

  [[nodiscard]] ActionServerHandle create_action_server(const char *action_name,
                                                        ActionHandler handler,
                                                        const CallbackGroup *group,
                                                        int32_t qos_depth) {
    action_cbs_.push_back(std::make_unique<ActionHandler>(std::move(handler)));
    ActionHandler *held = action_cbs_.back().get();
    return ActionServerHandle(
        n_, static_cast<RobotBusActionServerHandle *>(check_ptr(
                robot_bus_node_create_action_server_with_qos(
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
                    held, group ? group->raw() : nullptr, qos_depth),
                "create_action_server")));
  }

  ActionClient create_action_client(const char *action_name) {
    return create_action_client(action_name, 0);
  }

  ActionClient create_action_client(const char *action_name, int32_t qos_depth) {
    return ActionClient(static_cast<RobotBusActionClient *>(check_ptr(
        robot_bus_node_create_action_client_with_qos(n_, action_name, qos_depth),
        "create_action_client")));
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
