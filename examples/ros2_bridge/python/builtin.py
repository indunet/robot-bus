"""Built-in Ros2Bridge mappers: String topic, Trigger service, Fibonacci action.

Requires a sourced ROS 2 distro (Humble/Jazzy), ``rclpy``, and a running broker.

For the more common *custom* mapper pattern, see ``custom_add_two_ints.py``.
"""

from __future__ import annotations

import robot_bus
from robot_bus.ros2_bridge import (
    Direction,
    FibonacciActionMapper,
    Ros2Bridge,
    StdMsgsStringMapper,
    TriggerServiceMapper,
)


def main() -> None:
    if not robot_bus.ros2_available():
        raise SystemExit(
            "ROS 2 not available: source /opt/ros/humble|jazzy/setup.bash "
            "and install rclpy (just python-dev-ros2)"
        )

    bridge = (
        Ros2Bridge.new("examples_ros2_bridge_builtin")
        .bus_tcp("localhost")
        .route("/examples/chatter", "/examples/chatter")
        .mapper(StdMsgsStringMapper())
        .direction(Direction.Ros2ToBus)
        .add()
        .service("/examples/reset", "/examples/reset")
        .mapper(TriggerServiceMapper())
        .add()
        .action("/examples/fibonacci", "/examples/fibonacci")
        .mapper(FibonacciActionMapper())
        .add()
        .build()
    )
    print(
        "builtin bridge: /examples/chatter, /examples/reset, /examples/fibonacci "
        "(Ros2ToBus; Ctrl+C to stop)"
    )
    bridge.spin()


if __name__ == "__main__":
    main()
