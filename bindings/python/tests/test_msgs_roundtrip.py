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


def test_action_types_that_depend_on_geometry_msgs() -> None:
    """Action types that embed Pose2D must import ``robot_bus.geometry_msgs``.

    If codegen leaves ``from geometry_msgs...`` in ``*_pb2.py``, importing
    ``robot_bus.robot_bus_interfaces.action.v1`` raises
    ``No module named 'geometry_msgs'`` (interop python→cpp action on the
    old combined action package).
    """
    from robot_bus.example_interfaces.action.v1 import FibonacciGoal
    from robot_bus.geometry_msgs.msg.v1 import Pose2D
    from robot_bus.robot_bus_interfaces.action.v1 import PointNavigationGoal

    assert FibonacciGoal(order=5).order == 5
    goal = PointNavigationGoal(pose=Pose2D(x=1.0, y=2.0, theta=0.3))
    decoded = PointNavigationGoal()
    decoded.ParseFromString(goal.SerializeToString())
    assert decoded.pose.x == 1.0
    assert decoded.pose.y == 2.0
    assert PointNavigationGoal.__module__.startswith("robot_bus.robot_bus_interfaces.")


def test_generated_pb2_does_not_import_top_level_ros_packages() -> None:
    from pathlib import Path
    import re

    import robot_bus

    pattern = re.compile(
        r"^(?:from|import) ("
        r"ackermann_msgs|apriltag_msgs|builtin_interfaces|control_msgs|"
        r"diagnostic_msgs|example_interfaces|foxglove_msgs|geometry_msgs|"
        r"lifecycle_msgs|map_msgs|nav2_msgs|nav_msgs|sensor_msgs|shape_msgs|"
        r"std_msgs|std_srvs|tf2_msgs|trajectory_msgs|unique_identifier_msgs|"
        r"vision_msgs|visualization_msgs"
        r")\b"
    )
    root = Path(robot_bus.__file__).resolve().parent
    bad: list[str] = []
    for path in root.rglob("*_pb2.py"):
        for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if pattern.match(line):
                bad.append(f"{path.relative_to(root)}:{i}: {line}")
    assert not bad, "generated pb2 still imports top-level ROS packages:\n" + "\n".join(
        bad
    )


if __name__ == "__main__":
    test_imu_roundtrip()
    test_header_and_string()
    test_imports_under_robot_bus_namespace()
    test_action_types_that_depend_on_geometry_msgs()
    test_generated_pb2_does_not_import_top_level_ros_packages()
    print("python msgs smoke ok")
