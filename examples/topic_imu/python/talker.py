"""Publish a few sensor_msgs Imu messages on /examples/imu."""

from __future__ import annotations

import time

import robot_bus
from robot_bus.geometry_msgs.msg.v1 import Vector3
from robot_bus.sensor_msgs.msg.v1 import Imu


def main() -> None:
    node = robot_bus.Node("examples_imu_talker")
    pub = node.create_publisher("/examples/imu", Imu)
    time.sleep(0.3)  # ZMQ slow joiner

    for i in range(5):
        imu = Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8 + i * 0.01))
        pub.publish(imu)
        print(f"published Imu #{i}")
        time.sleep(0.2)


if __name__ == "__main__":
    main()
