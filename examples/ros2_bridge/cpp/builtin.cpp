// Built-in Ros2Bridge mappers: String topic, Trigger service, Fibonacci action.
//
//   just examples-cpp-ros2
//   ./examples/build/ros2_bridge_builtin
//
// For the more common *custom* mapper pattern, see ros2_bridge_custom_add_two_ints.
#include <robot_bus/ros2_bridge.hpp>

#include <iostream>

int main() {
  if (!robot_bus::ros2_available()) {
    std::cerr << "ROS 2 bridge not linked (need robot_bus_ros2_bridge / "
                 "just examples-cpp-ros2)\n";
    return 1;
  }

  auto bridge = robot_bus::Ros2Bridge::New("examples_ros2_bridge_builtin")
                    .bus_tcp("localhost")
                    .from_ros("/examples/chatter", robot_bus::TopicQos::ros_default())
                    .to_bus("/examples/chatter", robot_bus::TopicQos::bus())
                    .mapper(robot_bus::StdMsgsStringMapper{})
                    .add()
                    .service()
                    .from_ros("/examples/reset", robot_bus::TopicQos::ros_default())
                    .to_bus("/examples/reset", robot_bus::TopicQos::bus())
                    .mapper(robot_bus::TriggerServiceMapper{})
                    .add()
                    .action()
                    .from_ros("/examples/fibonacci", robot_bus::TopicQos::ros_default())
                    .to_bus("/examples/fibonacci", robot_bus::TopicQos::bus())
                    .mapper(robot_bus::FibonacciActionMapper{})
                    .add()
                    .build();

  std::cout << "builtin bridge: /examples/chatter, /examples/reset, "
               "/examples/fibonacci (Ros2ToBus; Ctrl+C to stop)\n";
  bridge.spin();
  return 0;
}
