// Action server for /examples/fibonacci.
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>

#include <iostream>
#include <string>
#include <utility>
#include <vector>

int main() {
  robot_bus::Node node("examples_fibonacci_server");
  auto act = robot_bus::create_action_server<example_interfaces::action::v1::FibonacciGoal>(
      node, "/examples/fibonacci",
      [](const example_interfaces::action::v1::FibonacciGoal &goal) {
        const int order = goal.order() < 0 ? 0 : goal.order();
        std::vector<int32_t> seq;
        seq.reserve(static_cast<size_t>(order));
        for (int i = 0; i < order; ++i) {
          if (i < 2) {
            seq.push_back(i);
          } else {
            seq.push_back(seq[static_cast<size_t>(i - 1)] + seq[static_cast<size_t>(i - 2)]);
          }
        }

        example_interfaces::action::v1::FibonacciFeedback feedback;
        for (size_t i = 0; i + 1 < seq.size(); ++i) {
          feedback.add_sequence(seq[i]);
        }
        example_interfaces::action::v1::FibonacciResult result;
        for (int32_t v : seq) {
          result.add_sequence(v);
        }

        std::vector<std::pair<std::string, std::vector<uint8_t>>> phases;
        phases.emplace_back("FEEDBACK", robot_bus::encode_pb(feedback));
        phases.emplace_back("RESULT", robot_bus::encode_pb(result));
        return phases;
      });
  (void)act;
  std::cout << "serving /examples/fibonacci (Ctrl+C to stop)\n";
  node.start();
  node.spin();
  return 0;
}
