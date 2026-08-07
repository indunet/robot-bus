// Built-in Fibonacci action client round-trip.
#include "harness.hpp"

#include <robot_bus/robot_bus_interface/action/v1/fibonacci.pb.h>

#include <iostream>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

int main() {
  using namespace robot_bus::test;
  static_assert(!std::is_copy_constructible_v<robot_bus::ActionGoalHandle>);
  static_assert(!std::is_copy_assignable_v<robot_bus::ActionGoalHandle>);
  static_assert(std::is_move_constructible_v<robot_bus::ActionGoalHandle>);
  static_assert(std::is_move_assignable_v<robot_bus::ActionGoalHandle>);

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

  std::vector<robot_bus::ActionMessage> feedback_messages;
  auto original_handle = client.send_goal(
      goal_bytes,
      [&](const robot_bus::ActionMessage &message) { feedback_messages.push_back(message); },
      "cpp-fibonacci-goal", 10.0);
  auto handle = std::move(original_handle);

  ROBOT_BUS_CHECK(handle.goal_id() == "cpp-fibonacci-goal");
  ROBOT_BUS_CHECK(handle.action_name() == "fibonacci");
  auto result_message = handle.wait_result(10.0);

  ROBOT_BUS_CHECK(feedback_messages.size() == 1);
  ROBOT_BUS_CHECK(feedback_messages[0].kind == "FEEDBACK");
  ROBOT_BUS_CHECK(feedback_messages[0].goal_id == "cpp-fibonacci-goal");
  ROBOT_BUS_CHECK(result_message.kind == "RESULT");
  ROBOT_BUS_CHECK(result_message.goal_id == "cpp-fibonacci-goal");

  robot_bus_interface::action::v1::FibonacciFeedback feedback;
  ROBOT_BUS_CHECK(feedback.ParseFromArray(feedback_messages[0].body.data(),
                                          static_cast<int>(feedback_messages[0].body.size())));
  ROBOT_BUS_CHECK(feedback.sequence_size() == 4);
  ROBOT_BUS_CHECK(feedback.sequence(0) == 0);
  ROBOT_BUS_CHECK(feedback.sequence(3) == 2);

  robot_bus_interface::action::v1::FibonacciResult result;
  ROBOT_BUS_CHECK(result.ParseFromArray(result_message.body.data(),
                                        static_cast<int>(result_message.body.size())));
  ROBOT_BUS_CHECK(result.sequence_size() == 5);
  ROBOT_BUS_CHECK(result.sequence(4) == 3);

  server.shutdown();
  server.wait();
  bus.stop();
  std::cout << "ok: action_fibonacci\n";
  return 0;
}
