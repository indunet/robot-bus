#ifndef ROBOT_BUS_H
#define ROBOT_BUS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
#ifdef ROBOT_BUS_BUILD
#define ROBOT_BUS_API __declspec(dllexport)
#else
#define ROBOT_BUS_API __declspec(dllimport)
#endif
#else
#define ROBOT_BUS_API __attribute__((visibility("default")))
#endif

typedef struct RobotBusPublisher RobotBusPublisher;
typedef struct RobotBusSubscriber RobotBusSubscriber;
typedef struct RobotBusShutdownHandle RobotBusShutdownHandle;
typedef struct RobotBusTimerHandle RobotBusTimerHandle;
typedef struct RobotBusSubscriptionHandle RobotBusSubscriptionHandle;
typedef struct RobotBusServiceHandle RobotBusServiceHandle;
typedef struct RobotBusActionServerHandle RobotBusActionServerHandle;
typedef struct RobotBusCallbackGroup RobotBusCallbackGroup;
typedef struct RobotBusTopicPublisher RobotBusTopicPublisher;
typedef struct RobotBusServiceClient RobotBusServiceClient;
typedef struct RobotBusActionClient RobotBusActionClient;
typedef struct RobotBusActionGoalHandle RobotBusActionGoalHandle;
typedef struct RobotBusNode RobotBusNode;
typedef struct RobotBusSingleThreadedExecutor RobotBusSingleThreadedExecutor;
typedef struct RobotBusMultiThreadedExecutor RobotBusMultiThreadedExecutor;
typedef struct RobotBusBroker RobotBusBroker;
typedef struct RobotBusContext RobotBusContext;

typedef struct RobotBusActionMessage {
  char *kind;
  uint8_t *body;
  size_t body_len;
  char *goal_id;
  char *action_name;
} RobotBusActionMessage;

typedef struct RobotBusActionPhase {
  char *phase;
  uint8_t *body;
  size_t body_len;
} RobotBusActionPhase;

typedef struct RobotBusNodeOptions {
  const char *host;
  const char *transport;
  const char *ws_url;
  const char *message_xsub;
  const char *message_xpub;
  const char *service_frontend;
  const char *service_backend;
  const char *action_backend;
  const char *action_frontend;
} RobotBusNodeOptions;

typedef struct RobotBusBrokerOptions {
  const char *message_xsub_bind;
  const char *message_xpub_bind;
  const char *service_frontend_bind;
  const char *service_backend_bind;
  const char *action_frontend_bind;
  const char *action_backend_bind;
  const char *api_listen;
  const char *console_listen;
  int tcp_only;
  int no_console;
  /** Hop-path id for federation (NULL / empty → random UUID at start). */
  const char *broker_id;
  /** Peer XPUB endpoints; XSUB derived as port - 1. Length: message_peer_count. */
  const char *const *message_peers;
  size_t message_peer_count;
  /** Peer service backends (`[id=]tcp://host:port`). Length: service_peer_count. */
  const char *const *service_peers;
  size_t service_peer_count;
  /** Peer action backends (`[id=]tcp://host:port`). Length: action_peer_count. */
  const char *const *action_peers;
  size_t action_peer_count;
  /** When non-zero, compatibility no-op (UDP announce removed). */
  int no_discovery;
  /** Soft label returned in /api/v1/discover (default 0). */
  uint32_t domain_id;
  /** Host clients should connect to (NULL → inferred). */
  const char *advertise_host;
  /** Federation peers as API URLs / host:port (GET /api/v1/discover). */
  const char *const *peers;
  size_t peer_count;
  /** When non-zero, hide tank demo and reject /api/v1/tank/session. */
  int no_tank;
} RobotBusBrokerOptions;

/** Client HTTP discovery options (NULL fields / 0 → defaults). */
typedef struct RobotBusDiscoverOpts {
  /** Broker API base URL, e.g. http://127.0.0.1:15570 (NULL → default). */
  const char *api_url;
  const char *broker_id;
  double timeout_secs;
} RobotBusDiscoverOpts;

/**
 * Owned endpoints after discovery + transport apply.
 * Free with robot_bus_applied_node_options_free.
 */
typedef struct RobotBusAppliedNodeOptions {
  char *host;
  char *transport;
  char *ws_url;
  char *message_xsub;
  char *message_xpub;
  char *service_frontend;
  char *service_backend;
  char *action_backend;
  char *action_frontend;
} RobotBusAppliedNodeOptions;

/** Parameter scalar type: 0=bool, 1=integer, 2=double, 3=string. */
typedef enum RobotBusParameterType {
  ROBOT_BUS_PARAM_BOOL = 0,
  ROBOT_BUS_PARAM_INTEGER = 1,
  ROBOT_BUS_PARAM_DOUBLE = 2,
  ROBOT_BUS_PARAM_STRING = 3
} RobotBusParameterType;

/**
 * Parameter value.
 * For declare/set: string_value is borrowed (not freed by the library).
 * For get/list: string_value is owned when type==STRING (free with robot_bus_free_string).
 */
typedef struct RobotBusParameterValue {
  int type;
  int bool_value;
  int64_t integer_value;
  double double_value;
  char *string_value;
} RobotBusParameterValue;

typedef struct RobotBusParameter {
  char *name;
  RobotBusParameterValue value;
} RobotBusParameter;

typedef void (*RobotBusMsgCallback)(const char *topic, const uint8_t *data, size_t len,
                                    void *user);
typedef void (*RobotBusTimerCallback)(void *user);
typedef int (*RobotBusServiceHandler)(const uint8_t *data, size_t len, uint8_t **out_data,
                                      size_t *out_len, void *user);
typedef int (*RobotBusActionHandler)(const uint8_t *data, size_t len,
                                     RobotBusActionPhase **out_phases, size_t *out_count,
                                     void *user);
/**
 * Called synchronously on the goal's receive thread for each FEEDBACK event.
 * The message and its fields are borrowed and valid only for the duration of the callback.
 */
typedef void (*RobotBusActionFeedbackCallback)(const RobotBusActionMessage *message, void *user);

ROBOT_BUS_API const char *robot_bus_last_error(void);
/** Set thread-local last error (usable from C++ TopicMapper callbacks). */
ROBOT_BUS_API void robot_bus_set_error(const char *msg);
ROBOT_BUS_API void robot_bus_free_string(char *s);
ROBOT_BUS_API void robot_bus_free_bytes(uint8_t *data, size_t len);
ROBOT_BUS_API uint8_t *robot_bus_alloc_bytes(size_t len);
ROBOT_BUS_API char *robot_bus_dup_string(const char *s);
ROBOT_BUS_API RobotBusActionPhase *robot_bus_alloc_action_phases(size_t count);
ROBOT_BUS_API void robot_bus_action_messages_free(RobotBusActionMessage *msgs, size_t count);
ROBOT_BUS_API void robot_bus_action_phases_free(RobotBusActionPhase *phases, size_t count);

ROBOT_BUS_API int robot_bus_message_xsub_endpoint(const char *host, const char *transport,
                                                  char **out);
ROBOT_BUS_API int robot_bus_message_xpub_endpoint(const char *host, const char *transport,
                                                  char **out);

ROBOT_BUS_API RobotBusPublisher *robot_bus_publisher_new(const char *endpoint);
ROBOT_BUS_API void robot_bus_publisher_free(RobotBusPublisher *p);
ROBOT_BUS_API int robot_bus_publisher_publish(RobotBusPublisher *p, const char *topic,
                                              const uint8_t *data, size_t len);
ROBOT_BUS_API char *robot_bus_publisher_endpoint(const RobotBusPublisher *p);

ROBOT_BUS_API RobotBusSubscriber *robot_bus_subscriber_new(const char *endpoint);
ROBOT_BUS_API void robot_bus_subscriber_free(RobotBusSubscriber *s);
ROBOT_BUS_API int robot_bus_subscriber_subscribe(RobotBusSubscriber *s, const char *topic);
ROBOT_BUS_API int robot_bus_subscriber_unsubscribe(RobotBusSubscriber *s, const char *topic);
ROBOT_BUS_API int robot_bus_subscriber_receive(RobotBusSubscriber *s, double timeout_secs,
                                               char **out_topic, uint8_t **out_data,
                                               size_t *out_len);
ROBOT_BUS_API char *robot_bus_subscriber_endpoint(const RobotBusSubscriber *s);

ROBOT_BUS_API void robot_bus_shutdown_handle_free(RobotBusShutdownHandle *h);
ROBOT_BUS_API void robot_bus_shutdown_handle_shutdown(RobotBusShutdownHandle *h);
ROBOT_BUS_API int robot_bus_shutdown_handle_is_running(const RobotBusShutdownHandle *h);
ROBOT_BUS_API void robot_bus_timer_handle_free(RobotBusTimerHandle *h);
ROBOT_BUS_API void robot_bus_subscription_handle_free(RobotBusSubscriptionHandle *h);
ROBOT_BUS_API void robot_bus_service_handle_free(RobotBusServiceHandle *h);
ROBOT_BUS_API void robot_bus_action_server_handle_free(RobotBusActionServerHandle *h);
ROBOT_BUS_API void robot_bus_callback_group_free(RobotBusCallbackGroup *g);
ROBOT_BUS_API uint64_t robot_bus_callback_group_id(const RobotBusCallbackGroup *g);
ROBOT_BUS_API int robot_bus_callback_group_kind(const RobotBusCallbackGroup *g);

ROBOT_BUS_API void robot_bus_topic_publisher_free(RobotBusTopicPublisher *p);
ROBOT_BUS_API char *robot_bus_topic_publisher_topic(const RobotBusTopicPublisher *p);
ROBOT_BUS_API int robot_bus_topic_publisher_publish(RobotBusTopicPublisher *p, const uint8_t *data,
                                                    size_t len);

ROBOT_BUS_API void robot_bus_service_client_free(RobotBusServiceClient *c);
ROBOT_BUS_API char *robot_bus_service_client_service_name(const RobotBusServiceClient *c);
ROBOT_BUS_API int robot_bus_service_client_call(RobotBusServiceClient *c, const uint8_t *data,
                                                size_t len, double timeout_secs,
                                                uint8_t **out_data, size_t *out_len);
/** Returns 1 if console reports workers>0, else 0. */
ROBOT_BUS_API int robot_bus_service_client_service_is_ready(const RobotBusServiceClient *c);
/** Returns 1 if ready within timeout; timeout_secs < 0 waits forever. */
ROBOT_BUS_API int robot_bus_service_client_wait_for_service(const RobotBusServiceClient *c,
                                                            double timeout_secs);

ROBOT_BUS_API void robot_bus_action_client_free(RobotBusActionClient *c);
ROBOT_BUS_API char *robot_bus_action_client_action_name(const RobotBusActionClient *c);
/** Returns 1 if console reports workers>0, else 0. */
ROBOT_BUS_API int robot_bus_action_client_action_server_is_ready(const RobotBusActionClient *c);
/** Returns 1 if ready within timeout; timeout_secs < 0 waits forever. */
ROBOT_BUS_API int robot_bus_action_client_wait_for_action_server(const RobotBusActionClient *c,
                                                                 double timeout_secs);
ROBOT_BUS_API int robot_bus_action_client_send_goal(RobotBusActionClient *c, const uint8_t *data,
                                                    size_t len, const char *goal_id,
                                                    double timeout_secs,
                                                    RobotBusActionFeedbackCallback feedback,
                                                    void *user,
                                                    RobotBusActionGoalHandle **out_handle);
ROBOT_BUS_API void robot_bus_action_goal_handle_free(RobotBusActionGoalHandle *h);
ROBOT_BUS_API char *robot_bus_action_goal_handle_goal_id(const RobotBusActionGoalHandle *h);
ROBOT_BUS_API char *robot_bus_action_goal_handle_action_name(const RobotBusActionGoalHandle *h);
ROBOT_BUS_API int robot_bus_action_goal_handle_wait_result(RobotBusActionGoalHandle *h,
                                                           double timeout_secs,
                                                           RobotBusActionMessage *out_msg);
ROBOT_BUS_API int robot_bus_action_goal_handle_cancel(RobotBusActionGoalHandle *h);

ROBOT_BUS_API RobotBusContext *robot_bus_context_new(void);
ROBOT_BUS_API void robot_bus_context_free(RobotBusContext *c);
ROBOT_BUS_API RobotBusContext *robot_bus_context_clone(const RobotBusContext *c);

ROBOT_BUS_API RobotBusNode *robot_bus_node_new(const char *name, const RobotBusNodeOptions *opts);
ROBOT_BUS_API RobotBusNode *robot_bus_node_new_with_context(RobotBusContext *ctx, const char *name,
                                                            const RobotBusNodeOptions *opts);
ROBOT_BUS_API RobotBusNode *robot_bus_node_tcp(const char *name, const char *host);
ROBOT_BUS_API RobotBusNode *robot_bus_node_ipc(const char *name, const char *path);
ROBOT_BUS_API RobotBusNode *robot_bus_node_inproc(const char *name, const char *prefix);
ROBOT_BUS_API RobotBusNode *robot_bus_node_inproc_with_context(RobotBusContext *ctx, const char *name,
                                                              const char *prefix);
ROBOT_BUS_API RobotBusNode *robot_bus_node_ws(const char *name);
ROBOT_BUS_API RobotBusNode *robot_bus_node_ws_at(const char *name, const char *url);
/**
 * Discover a broker via HTTP GET /api/v1/discover, apply `transport` (`tcp`/`ipc`/`inproc`/`grpc`),
 * and create a node. `opts` may be NULL (domain 0, default timeout).
 */
ROBOT_BUS_API RobotBusNode *robot_bus_node_discover(const char *name, const char *transport,
                                                    const RobotBusDiscoverOpts *opts);
/**
 * Discover + apply into owned endpoint strings (`out` must be zeroed by the caller).
 * Returns 0 on success.
 */
ROBOT_BUS_API int robot_bus_discover_node_options(const char *transport,
                                                  const RobotBusDiscoverOpts *opts,
                                                  RobotBusAppliedNodeOptions *out);
ROBOT_BUS_API void robot_bus_applied_node_options_free(RobotBusAppliedNodeOptions *o);
ROBOT_BUS_API void robot_bus_node_free(RobotBusNode *n);
ROBOT_BUS_API char *robot_bus_node_name(const RobotBusNode *n);
ROBOT_BUS_API RobotBusCallbackGroup *robot_bus_node_create_callback_group(RobotBusNode *n,
                                                                          int kind);
ROBOT_BUS_API RobotBusTopicPublisher *robot_bus_node_create_publisher(RobotBusNode *n,
                                                                      const char *topic);
/** depth <= 0 keeps default HWM; depth > 0 maps to KeepLast depth. */
ROBOT_BUS_API RobotBusTopicPublisher *robot_bus_node_create_publisher_with_qos(
    RobotBusNode *n, const char *topic, int32_t depth);
/** Returns opaque handle (NULL on error). Caller may free with
 * robot_bus_subscription_handle_free or destroy via robot_bus_node_destroy_subscription. */
ROBOT_BUS_API RobotBusSubscriptionHandle *robot_bus_node_create_subscription(
    RobotBusNode *n, const char *topic, RobotBusMsgCallback callback, void *user,
    const RobotBusCallbackGroup *group);
/** depth <= 0 keeps default HWM; depth > 0 maps to KeepLast depth. */
ROBOT_BUS_API RobotBusSubscriptionHandle *robot_bus_node_create_subscription_with_qos(
    RobotBusNode *n, const char *topic, RobotBusMsgCallback callback, void *user,
    const RobotBusCallbackGroup *group, int32_t depth);
ROBOT_BUS_API int robot_bus_node_destroy_subscription(RobotBusNode *n,
                                                      RobotBusSubscriptionHandle *handle);
ROBOT_BUS_API RobotBusTimerHandle *robot_bus_node_create_timer(RobotBusNode *n, double period_secs,
                                                               RobotBusTimerCallback callback,
                                                               void *user,
                                                               const RobotBusCallbackGroup *group);
ROBOT_BUS_API int robot_bus_node_cancel_timer(RobotBusNode *n, const RobotBusTimerHandle *handle);
/** Returns opaque handle (NULL on error). */
ROBOT_BUS_API RobotBusServiceHandle *robot_bus_node_create_service(
    RobotBusNode *n, const char *service_name, RobotBusServiceHandler handler, void *user,
    const RobotBusCallbackGroup *group);
ROBOT_BUS_API int robot_bus_node_destroy_service(RobotBusNode *n, RobotBusServiceHandle *handle);
ROBOT_BUS_API char *robot_bus_service_handle_name(const RobotBusServiceHandle *h);
ROBOT_BUS_API RobotBusServiceClient *robot_bus_node_create_client(RobotBusNode *n,
                                                                  const char *service_name);
/** Returns opaque handle (NULL on error). */
ROBOT_BUS_API RobotBusActionServerHandle *robot_bus_node_create_action_server(
    RobotBusNode *n, const char *action_name, RobotBusActionHandler handler, void *user,
    const RobotBusCallbackGroup *group);
ROBOT_BUS_API int robot_bus_node_destroy_action_server(RobotBusNode *n,
                                                       RobotBusActionServerHandle *handle);
ROBOT_BUS_API char *robot_bus_action_server_handle_name(const RobotBusActionServerHandle *h);
ROBOT_BUS_API RobotBusActionClient *robot_bus_node_create_action_client(RobotBusNode *n,
                                                                        const char *action_name);
ROBOT_BUS_API int robot_bus_node_connect_action_client(RobotBusNode *n);
ROBOT_BUS_API RobotBusShutdownHandle *robot_bus_node_shutdown_handle(RobotBusNode *n);
ROBOT_BUS_API int robot_bus_node_shutdown(RobotBusNode *n);
ROBOT_BUS_API int robot_bus_node_spin_once(RobotBusNode *n, double timeout_secs);
/**
 * Wait for one message on `topic`. Returns 1 and fills out_* on success,
 * 0 on timeout, negative on error. Caller frees out_data with robot_bus_free_bytes.
 * timeout_secs < 0 waits forever.
 */
ROBOT_BUS_API int robot_bus_node_wait_for_message(RobotBusNode *n, const char *topic,
                                                  double timeout_secs, uint8_t **out_data,
                                                  size_t *out_len);
ROBOT_BUS_API int robot_bus_node_spin(RobotBusNode *n);
ROBOT_BUS_API int robot_bus_node_start(RobotBusNode *n);
ROBOT_BUS_API int robot_bus_node_stop(RobotBusNode *n);
ROBOT_BUS_API int robot_bus_node_wait(RobotBusNode *n);

ROBOT_BUS_API int robot_bus_node_declare_parameter(RobotBusNode *n, const char *name,
                                                   const RobotBusParameterValue *value);
ROBOT_BUS_API int robot_bus_node_set_parameter(RobotBusNode *n, const char *name,
                                               const RobotBusParameterValue *value);
ROBOT_BUS_API int robot_bus_node_get_parameter(RobotBusNode *n, const char *name,
                                               RobotBusParameterValue *out);
ROBOT_BUS_API int robot_bus_node_has_parameter(const RobotBusNode *n, const char *name);
ROBOT_BUS_API int robot_bus_node_undeclare_parameter(RobotBusNode *n, const char *name);
/**
 * ROS-shaped list: fills `out_names` / `out_prefixes` (NUL-terminated string arrays).
 * `prefixes` may be NULL when `prefix_count` is 0. `depth` 0 = recursive.
 * Caller frees with `robot_bus_string_list_free`.
 */
ROBOT_BUS_API int robot_bus_node_list_parameters(RobotBusNode *n, const char *const *prefixes,
                                                size_t prefix_count, uint64_t depth,
                                                char ***out_names, size_t *out_names_count,
                                                char ***out_prefixes, size_t *out_prefixes_count);
/** Convenience: all parameters with values (name + value). */
ROBOT_BUS_API int robot_bus_node_list_all_parameters(RobotBusNode *n, RobotBusParameter **out,
                                                    size_t *out_count);
ROBOT_BUS_API void robot_bus_parameters_free(RobotBusParameter *params, size_t count);
ROBOT_BUS_API void robot_bus_string_list_free(char **list, size_t count);
ROBOT_BUS_API int robot_bus_node_load_parameters_from_yaml(RobotBusNode *n, const char *path);
ROBOT_BUS_API int robot_bus_node_load_parameters_from_yaml_str(RobotBusNode *n, const char *yaml);

ROBOT_BUS_API RobotBusSingleThreadedExecutor *robot_bus_single_threaded_executor_new(void);
ROBOT_BUS_API RobotBusSingleThreadedExecutor *robot_bus_single_threaded_executor_new_with_context(
    RobotBusContext *ctx);
ROBOT_BUS_API void robot_bus_single_threaded_executor_free(RobotBusSingleThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_single_threaded_executor_add_node(RobotBusSingleThreadedExecutor *e,
                                                              RobotBusNode *n);
ROBOT_BUS_API RobotBusNode *robot_bus_single_threaded_executor_create_node(
    RobotBusSingleThreadedExecutor *e, const char *name, const RobotBusNodeOptions *opts);
ROBOT_BUS_API RobotBusShutdownHandle *robot_bus_single_threaded_executor_shutdown_handle(
    RobotBusSingleThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_single_threaded_executor_shutdown(RobotBusSingleThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_single_threaded_executor_spin_once(RobotBusSingleThreadedExecutor *e,
                                                               double timeout_secs);
ROBOT_BUS_API int robot_bus_single_threaded_executor_spin(RobotBusSingleThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_single_threaded_executor_start(RobotBusSingleThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_single_threaded_executor_stop(RobotBusSingleThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_single_threaded_executor_wait(RobotBusSingleThreadedExecutor *e);

ROBOT_BUS_API RobotBusMultiThreadedExecutor *robot_bus_multi_threaded_executor_new(
    size_t num_threads);
ROBOT_BUS_API RobotBusMultiThreadedExecutor *robot_bus_multi_threaded_executor_new_with_context(
    RobotBusContext *ctx, size_t num_threads);
ROBOT_BUS_API void robot_bus_multi_threaded_executor_free(RobotBusMultiThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_multi_threaded_executor_add_node(RobotBusMultiThreadedExecutor *e,
                                                             RobotBusNode *n);
ROBOT_BUS_API RobotBusNode *robot_bus_multi_threaded_executor_create_node(
    RobotBusMultiThreadedExecutor *e, const char *name, const RobotBusNodeOptions *opts);
ROBOT_BUS_API RobotBusShutdownHandle *robot_bus_multi_threaded_executor_shutdown_handle(
    RobotBusMultiThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_multi_threaded_executor_shutdown(RobotBusMultiThreadedExecutor *e);
ROBOT_BUS_API int robot_bus_multi_threaded_executor_spin_once(RobotBusMultiThreadedExecutor *e,
                                                              double timeout_secs);
ROBOT_BUS_API int robot_bus_multi_threaded_executor_spin(RobotBusMultiThreadedExecutor *e);

ROBOT_BUS_API RobotBusBroker *robot_bus_broker_start(const RobotBusBrokerOptions *opts);
ROBOT_BUS_API RobotBusBroker *robot_bus_broker_start_with_context(RobotBusContext *ctx,
                                                                  const RobotBusBrokerOptions *opts);
ROBOT_BUS_API void robot_bus_broker_free(RobotBusBroker *b);
ROBOT_BUS_API int robot_bus_broker_stop(RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_message_xsub_bind(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_message_xpub_bind(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_service_frontend_bind(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_service_backend_bind(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_action_frontend_bind(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_action_backend_bind(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_api_listen(const RobotBusBroker *b);
ROBOT_BUS_API char *robot_bus_broker_console_listen(const RobotBusBroker *b);

/**
 * Deprecated: always returns 0. The C++ ROS 2 bridge is native rclcpp
 * (`robot_bus_ros2_bridge` / `ROBOT_BUS_HAS_ROS2`); use `robot_bus::ros2_available()`.
 */
ROBOT_BUS_API int robot_bus_ros2_available(void);


#ifdef __cplusplus
}
#endif

#endif /* ROBOT_BUS_H */
