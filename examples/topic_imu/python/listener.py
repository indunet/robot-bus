"""Subscribe to /examples/imu (typed sensor_msgs Imu).

Requires a running robot-bus-broker and an installed robot_bus package
(`pip install robot-bus` or `just python-dev`).
"""

from __future__ import annotations

import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu


def on_imu(imu: Imu) -> None:
    z = imu.linear_acceleration.z if imu.linear_acceleration else 0.0
    print(f"linear_acceleration.z={z}")


def main() -> None:
    node = robot_bus.Node("examples_imu_listener")
    node.create_subscription("/examples/imu", on_imu, msg_type=Imu)
    print("listening on /examples/imu (Ctrl+C to stop)")
    try:
        node.spin()
    except KeyboardInterrupt:
        node.shutdown()


if __name__ == "__main__":
    main()
