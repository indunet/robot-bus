// Call /examples/fibonacci once.
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>

#include <iostream>

int main() {
  robot_bus::Node node("examples_fibonacci_client");
  auto client =
      robot_bus::create_action_client<example_interfaces::action::v1::FibonacciGoal,
                                      example_interfaces::action::v1::FibonacciFeedback,
                                      example_interfaces::action::v1::FibonacciResult>(
          node, "/examples/fibonacci");
  node.start();
  client.wait_for_action_server(5.0);

  example_interfaces::action::v1::FibonacciGoal goal;
  goal.set_order(5);
  auto handle = client.send_goal(goal, [](const example_interfaces::action::v1::FibonacciFeedback &fb) {
    std::cout << "feedback:";
    for (int i = 0; i < fb.sequence_size(); ++i) {
      std::cout << " " << fb.sequence(i);
    }
    std::cout << "\n";
  });

  auto result = client.wait_result(handle, 10.0);
  std::cout << "result:";
  for (int i = 0; i < result.sequence_size(); ++i) {
    std::cout << " " << result.sequence(i);
  }
  std::cout << "\n";
  node.shutdown();
  return 0;
}
