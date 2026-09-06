#pragma once

#include <robot_bus/ros2_bridge_qos.hpp>

namespace robot_bus {

/// Topic codec. Builtins are ZSTs; custom mappers override `attach` (ROS2 builds).
class TopicMapper {
 public:
  virtual ~TopicMapper() = default;
  /// Builtins and [`TypedTopicMapper`] support opt-in `.lazy()`. Attach-only
  /// custom mappers return false (`.lazy().add()` throws).
  virtual bool supports_lazy() const { return false; }
#ifdef ROBOT_BUS_HAS_ROS2
  /// Wire this topic. Default throws — builtins use the library path instead.
  virtual void attach(TopicWireContext &ctx);
  /// Create a ROS subscription that forwards onto `bus_pub`. Used by `.lazy()`.
  virtual rclcpp::SubscriptionBase::SharedPtr create_ros2_to_bus_subscription(
      rclcpp::Node::SharedPtr ros_node, const std::string &ros_topic,
      std::shared_ptr<TopicPublisher> bus_pub, std::shared_ptr<std::mutex> mtx,
      const rclcpp::QoS &qos, std::shared_ptr<DropStats> drop_stats = {},
      std::shared_ptr<RouteHealth> health = {});
#endif
};

/// Service codec. Custom: override `attach` with concrete `create_service<T>`.
class ServiceMapper {
 public:
  virtual ~ServiceMapper() = default;
  virtual const char *type_name() const = 0;
#ifdef ROBOT_BUS_HAS_ROS2
  virtual void attach(ServiceWireContext &ctx);
#endif
};

/// Action codec. Custom: override `attach` with concrete action types.
class ActionMapper {
 public:
  virtual ~ActionMapper() = default;
  virtual const char *type_name() const = 0;
#ifdef ROBOT_BUS_HAS_ROS2
  virtual void attach(ActionWireContext &ctx);
#endif
};

/// Builtin: `std_msgs/msg/String` ↔ bus `std_msgs.msg.v1.String`.
struct StdMsgsStringMapper : TopicMapper {};

/// Builtin: `sensor_msgs/msg/Image` ↔ bus `sensor_msgs.msg.v1.Image`.
struct SensorMsgsImageMapper : TopicMapper {};

/// Builtin: `std_srvs/srv/Trigger`.
struct TriggerServiceMapper : ServiceMapper {
  const char *type_name() const override { return "std_srvs/srv/Trigger"; }
};

/// Builtin: `std_srvs/srv/SetBool`.
struct SetBoolServiceMapper : ServiceMapper {
  const char *type_name() const override { return "std_srvs/srv/SetBool"; }
};

/// Builtin: `example_interfaces/action/Fibonacci`.
struct FibonacciActionMapper : ActionMapper {
  const char *type_name() const override { return "example_interfaces/action/Fibonacci"; }
};

}  // namespace robot_bus

#ifdef ROBOT_BUS_HAS_ROS2
#include <robot_bus/ros2_bridge_typed.hpp>
#endif
