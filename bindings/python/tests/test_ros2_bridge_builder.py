"""Minimal unit tests for Python Ros2Bridge builder (no ROS spin required)."""

from __future__ import annotations

import pytest


def test_builder_requires_mapper():
    from robot_bus.ros2_bridge import Ros2Bridge

    with pytest.raises(ValueError, match="mapper"):
        Ros2Bridge.new("t").route("/a", "/a").add()


def test_builder_accepts_concrete_mappers_without_build():
    # Importing mappers pulls ROS message packages; skip if unsourced.
    pytest.importorskip("std_msgs")
    pytest.importorskip("std_srvs")
    from robot_bus.ros2_bridge import (
        Ros2Bridge,
        StdMsgsStringMapper,
        TriggerServiceMapper,
    )

    b = (
        Ros2Bridge.new("t")
        .route("/chatter", "/chatter")
        .mapper(StdMsgsStringMapper())
        .add()
        .service("/reset", "/reset")
        .mapper(TriggerServiceMapper())
        .timeout(2.0)
        .add()
    )
    assert len(b._routes) == 1
    assert len(b._services) == 1


def test_ros2_available_checks_rclpy():
    import robot_bus

    try:
        import rclpy  # noqa: F401
    except ImportError:
        assert robot_bus.ros2_available() is False
    else:
        assert robot_bus.ros2_available() is True
