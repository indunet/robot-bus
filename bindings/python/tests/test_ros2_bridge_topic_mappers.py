"""Core distro topic mapper catalog (Humble/Jazzy common interfaces)."""

from __future__ import annotations

import re
from pathlib import Path

MAPPERS = Path(__file__).resolve().parents[1] / "robot_bus" / "ros2_bridge" / "mappers"
TYPE_NAME_RE = re.compile(r"`([a-z0-9_]+/msg/[A-Za-z0-9_]+)`")
EXTENSION_PREFIXES = (
    "foxglove_msgs/",
    "nav2_msgs/",
    "control_msgs/",
    "apriltag_msgs/",
)


def _type_names() -> set[str]:
    names: set[str] = set()
    for path in MAPPERS.rglob("*.py"):
        if path.name.startswith("_"):
            continue
        for match in TYPE_NAME_RE.finditer(path.read_text()):
            names.add(match.group(1))
    return names


def test_generated_topic_mapper_catalog():
    names = _type_names()
    assert 120 <= len(names) <= 130, f"expected ~125 core topic mappers, got {len(names)}"
    assert "geometry_msgs/msg/PoseStamped" in names
    assert "sensor_msgs/msg/Image" in names
    assert "std_msgs/msg/String" in names
    for name in names:
        assert not name.startswith(EXTENSION_PREFIXES), name


def test_pose_stamped_mapper_exists():
    from robot_bus.ros2_bridge.mappers import GeometryMsgsPoseStampedMapper

    assert GeometryMsgsPoseStampedMapper() is not None
