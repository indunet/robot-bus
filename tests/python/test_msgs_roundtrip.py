"""Smoke test: robot_bus message packages encode/decode (no broker required)."""

from __future__ import annotations

from robot_bus.geometry_msgs.msg.v1 import Vector3
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.std_msgs.msg.v1 import Header, String as ProtoString


def test_imu_roundtrip() -> None:
    imu = Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8))
    raw = imu.SerializeToString()
    decoded = Imu()
    decoded.ParseFromString(raw)
    assert decoded.linear_acceleration.z == 9.8


def test_header_and_string() -> None:
    header = Header(frame_id="base_link")
    msg = ProtoString(data="hello")
    assert Header.FromString(header.SerializeToString()).frame_id == "base_link"
    assert ProtoString.FromString(msg.SerializeToString()).data == "hello"


def test_imports_under_robot_bus_namespace() -> None:
    assert Imu.__module__.startswith("robot_bus.sensor_msgs.")
    assert Vector3.__module__.startswith("robot_bus.geometry_msgs.")


if __name__ == "__main__":
    test_imu_roundtrip()
    test_header_and_string()
    test_imports_under_robot_bus_namespace()
    print("python msgs smoke ok")
