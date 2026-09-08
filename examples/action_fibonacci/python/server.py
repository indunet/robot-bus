"""Action server for /examples/fibonacci (example_interfaces Fibonacci)."""

from __future__ import annotations

import robot_bus
from robot_bus.example_interfaces.action.v1 import (
    FibonacciFeedback,
    FibonacciGoal,
    FibonacciResult,
)


def on_fibonacci(goal: FibonacciGoal):
    order = max(goal.order, 0)
    seq: list[int] = []
    for i in range(order):
        if i < 2:
            seq.append(i)
        else:
            seq.append(seq[i - 1] + seq[i - 2])
    # Typed Python handlers return phase/message pairs. Feedback is emitted
    # after this handler returns; live raw handlers use streaming=True.
    return [
        ("FEEDBACK", FibonacciFeedback(sequence=seq[:-1] if len(seq) > 1 else seq)),
        ("RESULT", FibonacciResult(sequence=seq)),
    ]


def main() -> None:
    node = robot_bus.Node("examples_fibonacci_server")
    node.create_action_server(
        "/examples/fibonacci",
        on_fibonacci,
        goal_type=FibonacciGoal,
        feedback_type=FibonacciFeedback,
        result_type=FibonacciResult,
    )
    print("serving /examples/fibonacci (Ctrl+C to stop)")
    try:
        node.spin()
    except KeyboardInterrupt:
        node.shutdown()


if __name__ == "__main__":
    main()
