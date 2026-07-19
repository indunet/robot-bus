// Built-in Fibonacci action client round-trip.
#include "harness.hpp"

#include <robot_bus/robot_bus_interface/action/v1/fibonacci.pb.hpp>

#include <iostream>
#include <string>
#include <utility>
#include <vector>

int main() {
  using namespace robot_bus::test;

  auto bus = TestBus::start();
  auto server = bus.make_node("act_server");
  auto client_node = bus.make_node("act_client");

  server.create_action_server("fibonacci", [](robot_bus::BytesView body) {
    robot_bus_interface::action::v1::FibonacciGoal goal;
    ROBOT_BUS_CHECK(goal.ParseFromArray(body.data, static_cast<int>(body.size)));
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

    robot_bus_interface::action::v1::FibonacciFeedback feedback;
    for (size_t i = 0; i + 1 < seq.size(); ++i) {
      feedback.add_sequence(seq[i]);
    }
    if (seq.size() <= 1) {
      for (int32_t v : seq) {
        feedback.add_sequence(v);
      }
    }

    robot_bus_interface::action::v1::FibonacciResult result;
    for (int32_t v : seq) {
      result.add_sequence(v);
    }

    std::string fb_bytes;
    std::string res_bytes;
    ROBOT_BUS_CHECK(feedback.SerializeToString(&fb_bytes));
    ROBOT_BUS_CHECK(result.SerializeToString(&res_bytes));

    std::vector<std::pair<std::string, std::vector<uint8_t>>> phases;
    phases.emplace_back("FEEDBACK", std::vector<uint8_t>(fb_bytes.begin(), fb_bytes.end()));
    phases.emplace_back("RESULT", std::vector<uint8_t>(res_bytes.begin(), res_bytes.end()));
    return phases;
  });

  server.start();
  sleep_ms(150);

  auto client = client_node.create_action_client("fibonacci");
  robot_bus_interface::action::v1::FibonacciGoal goal;
  goal.set_order(5);
  std::string goal_bytes;
  ROBOT_BUS_CHECK(goal.SerializeToString(&goal_bytes));

  auto messages = client.send_goal(goal_bytes, nullptr, 10.0);
  ROBOT_BUS_CHECK(messages.size() == 2);
  ROBOT_BUS_CHECK(messages[0].kind == "FEEDBACK");
  ROBOT_BUS_CHECK(messages[1].kind == "RESULT");

  robot_bus_interface::action::v1::FibonacciFeedback feedback;
  ROBOT_BUS_CHECK(feedback.ParseFromArray(messages[0].body.data(),
                                          static_cast<int>(messages[0].body.size())));
  ROBOT_BUS_CHECK(feedback.sequence_size() == 4);
  ROBOT_BUS_CHECK(feedback.sequence(0) == 0);
  ROBOT_BUS_CHECK(feedback.sequence(3) == 2);

  robot_bus_interface::action::v1::FibonacciResult result;
  ROBOT_BUS_CHECK(result.ParseFromArray(messages[1].body.data(),
                                        static_cast<int>(messages[1].body.size())));
  ROBOT_BUS_CHECK(result.sequence_size() == 5);
  ROBOT_BUS_CHECK(result.sequence(4) == 3);

  server.shutdown();
  server.wait();
  bus.stop();
  std::cout << "ok: action_fibonacci\n";
  return 0;
}
