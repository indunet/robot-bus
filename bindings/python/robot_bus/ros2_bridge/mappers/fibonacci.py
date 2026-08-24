"""Builtin: `example_interfaces/action/Fibonacci`."""

from __future__ import annotations


class FibonacciActionMapper:
    def type_name(self) -> str:
        return "example_interfaces/action/Fibonacci"

    def ros_action_type(self):
        from example_interfaces.action import Fibonacci

        return Fibonacci

    def ros_goal_to_bus(self, goal) -> bytes:
        from robot_bus.example_interfaces.action.v1 import FibonacciGoal

        return FibonacciGoal(order=int(goal.order)).SerializeToString()

    def bus_goal_to_ros(self, payload: bytes):
        from example_interfaces.action import Fibonacci
        from robot_bus.example_interfaces.action.v1 import FibonacciGoal

        bus = FibonacciGoal()
        bus.ParseFromString(payload)
        out = Fibonacci.Goal()
        out.order = bus.order
        return out

    def ros_feedback_to_bus(self, feedback) -> bytes:
        from robot_bus.example_interfaces.action.v1 import FibonacciFeedback

        return FibonacciFeedback(sequence=list(feedback.sequence)).SerializeToString()

    def bus_feedback_to_ros(self, payload: bytes):
        from example_interfaces.action import Fibonacci
        from robot_bus.example_interfaces.action.v1 import FibonacciFeedback

        bus = FibonacciFeedback()
        bus.ParseFromString(payload)
        out = Fibonacci.Feedback()
        out.sequence = list(bus.sequence)
        return out

    def ros_result_to_bus(self, result) -> bytes:
        from robot_bus.example_interfaces.action.v1 import FibonacciResult

        return FibonacciResult(sequence=list(result.sequence)).SerializeToString()

    def bus_result_to_ros(self, payload: bytes):
        from example_interfaces.action import Fibonacci
        from robot_bus.example_interfaces.action.v1 import FibonacciResult

        bus = FibonacciResult()
        bus.ParseFromString(payload)
        out = Fibonacci.Result()
        out.sequence = list(bus.sequence)
        return out
