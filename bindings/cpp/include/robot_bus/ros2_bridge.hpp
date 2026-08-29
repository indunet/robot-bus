#pragma once

#include <robot_bus/ros2_bridge_builder.hpp>

namespace robot_bus {

/// In-process rclcpp ↔ robot-bus bridge (`robot_bus_ros2_bridge` / `ROBOT_BUS_HAS_ROS2`).
class Ros2Bridge {
 public:
  static Ros2BridgeBuilder New(std::string name) {
    return Ros2BridgeBuilder(std::move(name));
  }

#ifdef ROBOT_BUS_HAS_ROS2
  ~Ros2Bridge();
  Ros2Bridge(const Ros2Bridge &) = delete;
  Ros2Bridge &operator=(const Ros2Bridge &) = delete;
  Ros2Bridge(Ros2Bridge &&) noexcept;
  Ros2Bridge &operator=(Ros2Bridge &&) noexcept;

  void spin();
  void spin_once(double timeout_secs = 0.01);
  /// True when this bridge currently holds a ROS subscription for `bus_topic`.
  bool has_ros_subscription(const std::string &bus_topic) const;

 private:
  friend class Ros2BridgeBuilder;
  struct Impl;
  explicit Ros2Bridge(std::unique_ptr<Impl> impl);
  std::unique_ptr<Impl> impl_;
#else
  Ros2Bridge() = delete;
#endif
};

#ifndef ROBOT_BUS_HAS_ROS2
inline Ros2Bridge Ros2BridgeBuilder::build() && {
  (void)state_;
  throw Error(kRos2BridgeUnavailable);
}
#endif

}  // namespace robot_bus
