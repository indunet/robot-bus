"""Built-in Ros2Bridge mappers: String topic, Trigger service, Fibonacci action.

Requires a sourced ROS 2 distro (Humble/Jazzy), ``rclpy``, and a running broker.

For the more common *custom* mapper pattern, see ``custom_add_two_ints.py``.
"""

from __future__ import annotations

import robot_bus
from robot_bus.ros2_bridge import (
    FibonacciActionMapper,
    Ros2Bridge,
    StdMsgsStringMapper,
    TopicQos,
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
        .from_ros("/examples/chatter", TopicQos.default())
        .to_bus("/examples/chatter", TopicQos.bus())
        .mapper(StdMsgsStringMapper())
        .add()
        .service()
        .from_ros("/examples/reset", TopicQos.default())
        .to_bus("/examples/reset", TopicQos.bus())
        .mapper(TriggerServiceMapper())
        .add()
        .action()
        .from_ros("/examples/fibonacci", TopicQos.default())
        .to_bus("/examples/fibonacci", TopicQos.bus())
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
