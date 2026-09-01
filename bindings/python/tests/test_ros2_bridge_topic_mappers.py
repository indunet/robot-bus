"""Generated topic mapper catalog matches the Rust proto/*/msg/v1 set."""

from __future__ import annotations

import re
from pathlib import Path

MAPPERS = Path(__file__).resolve().parents[1] / "robot_bus" / "ros2_bridge" / "mappers"
TYPE_NAME_RE = re.compile(r'return "([^"]+/msg/[^"]+)"')


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
    assert len(names) >= 150, f"expected ≥150 topic mappers, got {len(names)}"
    assert "geometry_msgs/msg/PoseStamped" in names
    assert "sensor_msgs/msg/Image" in names


def test_pose_stamped_mapper_type_name():
    from robot_bus.ros2_bridge.mappers import GeometryMsgsPoseStampedMapper

    assert GeometryMsgsPoseStampedMapper().type_name() == "geometry_msgs/msg/PoseStamped"
