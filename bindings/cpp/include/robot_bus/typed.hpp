#pragma once

/// Header-only typed wrappers over protobuf `MessageLite` (mirrors Java/Python).
/// Requires linking `robot_bus_msgs` (`ROBOT_BUS_BUILD_MSGS=ON`).

#include <robot_bus/node.hpp>

#include <google/protobuf/message_lite.h>

#include <cstdio>
#include <functional>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

namespace robot_bus {

namespace detail {

inline std::vector<uint8_t> encode_message_lite(const google::protobuf::MessageLite &msg) {
  const int size = static_cast<int>(msg.ByteSizeLong());
  std::vector<uint8_t> bytes(static_cast<size_t>(size));
  if (size > 0 && !msg.SerializeToArray(bytes.data(), size)) {
    throw Error("protobuf SerializeToArray failed");
  }
  return bytes;
}

template <typename Msg>
bool try_parse_message_lite(const BytesView &payload, Msg *out) {
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Msg>,
                "Msg must derive from google::protobuf::MessageLite");
  if (!out) {
    return false;
  }
  if (!out->ParseFromArray(payload.data, static_cast<int>(payload.size))) {
    std::fprintf(stderr, "robot_bus typed decode failed\n");
    return false;
  }
  return true;
}

}  // namespace detail

/// Encode any MessageLite to opaque bytes (for typed action FEEDBACK/RESULT phases).
inline std::vector<uint8_t> encode_pb(const google::protobuf::MessageLite &msg) {
  return detail::encode_message_lite(msg);
}

template <typename Msg>
class TypedTopicPublisher {
 public:
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Msg>,
                "Msg must derive from google::protobuf::MessageLite");

  explicit TypedTopicPublisher(TopicPublisher inner) : inner_(std::move(inner)) {}

  std::string topic() const { return inner_.topic(); }

  void publish(const Msg &msg) { inner_.publish(detail::encode_message_lite(msg)); }

  TopicPublisher &raw() { return inner_; }
  const TopicPublisher &raw() const { return inner_; }

 private:
  TopicPublisher inner_;
};

template <typename Msg>
[[nodiscard]] TypedTopicPublisher<Msg> create_publisher(Node &node, const char *topic,
                                                        int32_t qos_depth = 0) {
  if (qos_depth > 0) {
    return TypedTopicPublisher<Msg>(node.create_publisher(topic, qos_depth));
  }
  return TypedTopicPublisher<Msg>(node.create_publisher(topic));
}

template <typename Msg>
[[nodiscard]] SubscriptionHandle create_subscription(
    Node &node, const char *topic, std::function<void(std::string_view, const Msg &)> callback,
    const CallbackGroup *group = nullptr, int32_t qos_depth = 0) {
  return node.create_subscription(
      topic,
      [cb = std::move(callback)](std::string_view t, BytesView payload) {
        Msg msg;
        if (!detail::try_parse_message_lite(payload, &msg)) {
          return;
        }
        cb(t, msg);
      },
      group, qos_depth);
}

template <typename Req, typename Resp>
class TypedServiceClient {
 public:
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Req>,
                "Req must derive from google::protobuf::MessageLite");
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Resp>,
                "Resp must derive from google::protobuf::MessageLite");

  explicit TypedServiceClient(ServiceClient inner) : inner_(std::move(inner)) {}

  std::string service_name() const { return inner_.service_name(); }
  bool service_is_ready() const { return inner_.service_is_ready(); }
  bool wait_for_service(double timeout_secs = -1.0) const {
    return inner_.wait_for_service(timeout_secs);
  }

  Resp call(const Req &request, double timeout_secs = -1.0) {
    auto reply = inner_.call(detail::encode_message_lite(request), timeout_secs);
    Resp out;
    if (!detail::try_parse_message_lite(BytesView(reply), &out)) {
      throw Error("typed service response decode failed");
    }
    return out;
  }

  ServiceClient &raw() { return inner_; }
  const ServiceClient &raw() const { return inner_; }

 private:
  ServiceClient inner_;
};

template <typename Req, typename Resp>
TypedServiceClient<Req, Resp> create_client(Node &node, const char *service_name,
                                            int32_t qos_depth = 0) {
  return TypedServiceClient<Req, Resp>(node.create_client(service_name, qos_depth));
}

template <typename Req, typename Resp>
[[nodiscard]] ServiceHandle create_service(Node &node, const char *service_name,
                             std::function<Resp(const Req &)> handler,
                             const CallbackGroup *group = nullptr, int32_t qos_depth = 0) {
  return node.create_service(
      service_name,
      [handler = std::move(handler)](BytesView body) {
        Req req;
        if (!detail::try_parse_message_lite(body, &req)) {
          return std::vector<uint8_t>{};
        }
        return detail::encode_message_lite(handler(req));
      },
      group, qos_depth);
}

template <typename Goal, typename Feedback, typename Result>
class TypedActionClient {
 public:
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Goal>,
                "Goal must derive from google::protobuf::MessageLite");
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Feedback>,
                "Feedback must derive from google::protobuf::MessageLite");
  static_assert(std::is_base_of_v<google::protobuf::MessageLite, Result>,
                "Result must derive from google::protobuf::MessageLite");

  explicit TypedActionClient(ActionClient inner) : inner_(std::move(inner)) {}

  std::string action_name() const { return inner_.action_name(); }
  bool action_server_is_ready() const { return inner_.action_server_is_ready(); }
  bool wait_for_action_server(double timeout_secs = -1.0) const {
    return inner_.wait_for_action_server(timeout_secs);
  }

  ActionGoalHandle send_goal(const Goal &goal, std::function<void(const Feedback &)> on_feedback = {},
                             const char *goal_id = nullptr, double timeout_secs = -1.0) {
    ActionFeedbackCallback fb;
    if (on_feedback) {
      fb = [on_feedback = std::move(on_feedback)](const ActionMessage &msg) {
        if (msg.kind != "FEEDBACK") {
          return;
        }
        Feedback decoded;
        if (!detail::try_parse_message_lite(BytesView(msg.body), &decoded)) {
          return;
        }
        on_feedback(decoded);
      };
    }
    return inner_.send_goal(detail::encode_message_lite(goal), std::move(fb), goal_id, timeout_secs);
  }

  Result wait_result(ActionGoalHandle &handle, double timeout_secs = -1.0) {
    ActionMessage msg = handle.wait_result(timeout_secs);
    Result out;
    if (!detail::try_parse_message_lite(BytesView(msg.body), &out)) {
      throw Error("typed action result decode failed");
    }
    return out;
  }

  ActionClient &raw() { return inner_; }
  const ActionClient &raw() const { return inner_; }

 private:
  ActionClient inner_;
};

template <typename Goal, typename Feedback, typename Result>
TypedActionClient<Goal, Feedback, Result> create_action_client(Node &node,
                                                               const char *action_name,
                                                               int32_t qos_depth = 0) {
  return TypedActionClient<Goal, Feedback, Result>(
      node.create_action_client(action_name, qos_depth));
}

/// Typed action server. Handler returns `(phase, body)` pairs; use `encode_pb(feedback|result)`.
template <typename Goal>
[[nodiscard]] ActionServerHandle create_action_server(
    Node &node, const char *action_name,
    std::function<std::vector<std::pair<std::string, std::vector<uint8_t>>>(const Goal &)> handler,
    const CallbackGroup *group = nullptr, int32_t qos_depth = 0) {
  return node.create_action_server(
      action_name,
      [handler = std::move(handler)](BytesView body) {
        Goal goal;
        if (!detail::try_parse_message_lite(body, &goal)) {
          return std::vector<std::pair<std::string, std::vector<uint8_t>>>{{"RESULT", {}}};
        }
        return handler(goal);
      },
      group, qos_depth);
}

}  // namespace robot_bus
