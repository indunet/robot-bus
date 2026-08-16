"""Call /examples/fibonacci once."""

from __future__ import annotations

import robot_bus
from robot_bus.example_interfaces.action.v1 import (
    FibonacciFeedback,
    FibonacciGoal,
    FibonacciResult,
)


def main() -> None:
    node = robot_bus.Node("examples_fibonacci_client")
    client = node.create_action_client(
        "/examples/fibonacci",
        goal_type=FibonacciGoal,
        feedback_type=FibonacciFeedback,
        result_type=FibonacciResult,
    )
    client.wait_for_action_server(timeout=5.0)
    goal = client.send_goal(
        FibonacciGoal(order=5),
        feedback_callback=lambda fb: print(f"feedback: {list(fb.sequence)}"),
    )
    result = goal.result(timeout=10.0)
    print(f"result: {list(result.sequence)}")


if __name__ == "__main__":
    main()
